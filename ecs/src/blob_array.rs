use std::{alloc, ptr, ptr::NonNull};

#[derive(Debug)]
pub struct BlobArray {
    item_layout: core::alloc::Layout,
    data: core::ptr::NonNull<u8>,
}

impl BlobArray {
    pub fn with_capacity(item_layout: core::alloc::Layout, capacity: usize) -> Self {
        let array_layout = item_layout.repeat_packed(capacity).unwrap();
        let data_allocation = unsafe { alloc::alloc(array_layout) };
        let data = ptr::NonNull::new(data_allocation).unwrap();

        Self { item_layout, data }
    }

    pub fn get_item(&self, index: usize) -> NonNull<u8> {
        unsafe { self.data.byte_add(self.item_layout.size() * index) }
    }

    pub fn place_at(&mut self, item: core::ptr::NonNull<u8>, index: usize) {
        unsafe {
            std::ptr::copy(
                item.as_ptr(),
                self.data.as_ptr().byte_add(self.item_layout.size() * index),
                self.item_layout.size(),
            );
        }
    }

    pub fn drop(&mut self, _size: usize, capacity: usize) {
        unsafe {
            alloc::dealloc(
                self.data.as_ptr().cast(),
                self.item_layout.repeat_packed(capacity).unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use core::alloc;
    use std::ptr;

    use crate::blob_array::BlobArray;

    #[derive(Debug)]
    struct PositionStruct {
        _potato: i64,
        l: u8,
        _chirrup: u32,
        _n: u8,
        _k: u8,
        _m: u8,
    }

    #[test]
    fn lala() {
        let layout = alloc::Layout::new::<PositionStruct>();
        let mut blobee = BlobArray::with_capacity(layout, 3);

        println!("Lemme see garbage: {:?}", unsafe {
            blobee.data.cast::<PositionStruct>().read()
        });

        let mut position_one = PositionStruct {
            _potato: 1,
            l: 2,
            _k: 4,
            _m: 4,
            _n: 4,
            _chirrup: 3,
        };
        let ptr =
            ptr::NonNull::new((&mut position_one as *mut PositionStruct).cast::<u8>()).unwrap();
        blobee.place_at(ptr, 0);

        let mut position_two = PositionStruct {
            _potato: 2,
            l: 3,
            _k: 4,
            _m: 4,
            _n: 4,
            _chirrup: 4,
        };
        let ptr =
            ptr::NonNull::new((&mut position_two as *mut PositionStruct).cast::<u8>()).unwrap();
        blobee.place_at(ptr, 1);

        let mut position_three = PositionStruct {
            _potato: 9,
            l: 9,
            _k: 4,
            _m: 4,
            _n: 4,
            _chirrup: 9,
        };
        let ptr =
            ptr::NonNull::new((&mut position_three as *mut PositionStruct).cast::<u8>()).unwrap();
        blobee.place_at(ptr, 2);

        println!("Lemme see 1: {:?}", unsafe {
            blobee.data.cast::<PositionStruct>().read()
        });
        println!("Lemme see 2: {:?}", unsafe {
            blobee
                .data
                .byte_add(blobee.item_layout.size())
                .cast::<PositionStruct>()
                .read()
        });
        println!("Lemme see 3: {:?}", unsafe {
            blobee
                .data
                .byte_add(blobee.item_layout.size() * 2)
                .cast::<PositionStruct>()
                .read()
        });
        unsafe {
            blobee
                .data
                .byte_add(blobee.item_layout.size() * 2)
                .cast::<PositionStruct>()
                .as_mut()
                .l = 12;
        };
        println!("Lemme see 3 after mod: {:?}", unsafe {
            blobee
                .data
                .byte_add(blobee.item_layout.size() * 2)
                .cast::<PositionStruct>()
                .read()
        });

        blobee.drop(3, 3);
    }
}
