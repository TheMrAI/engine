use std::{alloc, ptr, ptr::NonNull};

#[derive(Debug)]
pub struct Blob {
    item_layout: core::alloc::Layout,
    data: core::ptr::NonNull<u8>,
}

impl Blob {
    pub fn with_capacity(item_layout: core::alloc::Layout, capacity: usize) -> Self {
        // TODO change to repeat
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
            std::ptr::copy_nonoverlapping(
                item.as_ptr(),
                self.data.as_ptr().byte_add(self.item_layout.size() * index),
                self.item_layout.size(),
            );
        }
    }

    /// If the supplied capacity is less than the allocated capacity, then not even the address sanitizer may catch the memory leak.
    pub unsafe fn drop(
        &mut self,
        size: usize,
        capacity: usize,
        item_drop_fn: Option<unsafe fn(NonNull<u8>) -> ()>,
    ) {
        match item_drop_fn {
            Some(drop_fn) => {
                for index in 0..size {
                    unsafe {
                        drop_fn(self.get_item(index));
                    }
                }
            }
            None => {}
        }
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
    use std::alloc;
    use std::alloc::handle_alloc_error;
    use std::ptr::{self, NonNull};

    use crate::blob::Blob;

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
        let mut blobee = Blob::with_capacity(layout, 3);

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

        unsafe {
            blobee.drop(3, 3, None);
        }
    }

    #[test]
    fn vector_drop() {
        let layout = alloc::Layout::new::<Vec<u8>>();
        let position_one = unsafe { alloc::alloc(layout) };

        if position_one.is_null() {
            handle_alloc_error(layout);
        }
        unsafe {
            std::ptr::write(position_one as *mut Vec<u8>, vec![1, 2, 3]);
        }
        println!("Lemme see vector: {:?}", unsafe {
            &(*(position_one as *mut Vec<u8>))
        });

        unsafe {
            std::ptr::drop_in_place::<Vec<u8>>(position_one as *mut Vec<u8>);
        }
        unsafe { alloc::dealloc(position_one, layout) };
    }

    #[test]
    fn alloc_val() {
        let layout = alloc::Layout::new::<u64>();
        let ptr = unsafe { alloc::alloc(layout) };

        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        println!("Lemme see u64: {:?}", unsafe { &(*(ptr as *mut u64)) });
        unsafe { alloc::dealloc(ptr, layout) };
    }

    unsafe fn vec_u8_drop(item_ptr: NonNull<u8>) -> () {
        unsafe {
            std::ptr::drop_in_place::<Vec<u8>>(item_ptr.as_ptr() as *mut Vec<u8>);
        }
    }

    #[test]
    fn blobee_drop() {
        let layout = alloc::Layout::new::<Vec<u8>>();
        let mut blobee = Blob::with_capacity(layout, 3);

        let ptr = unsafe { alloc::alloc(layout) };
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        let data_ptr = unsafe {
            std::ptr::write(ptr as *mut Vec<u8>, vec![1, 2, 3]);
            NonNull::new(ptr).unwrap()
        };
        blobee.place_at(data_ptr, 0);

        println!("See from blobee: {:?}", unsafe {
            &*(blobee.get_item(0).as_ptr() as *mut Vec<u8>)
        });

        unsafe {
            blobee.drop(1, 3, Some(vec_u8_drop));
            std::alloc::dealloc(ptr, layout);
        }
    }
}
