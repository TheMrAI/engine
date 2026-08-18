use std::{
    alloc, cmp,
    marker::PhantomData,
    mem,
    num::{self},
    ptr::{self},
};

use crate::ComponentDropFn;

/// A `Blob` of memory
///
/// A type erased [Vec] without most safeties of a [Vec].
/// Given that the type is not known, stored elements cannot be
/// dropped.
/// Neither the capacity, nor the size of the underlying storage is known.
/// Managing these is the responsibility of the caller.
///
/// The [Blob] can allocates/deallocates the raw memory for storing the elements
/// themselves and provides methods for accessing/modifying these elements.
/// If the stored type requires a [drop] to be called, the caller must provide
/// such a function at all required times.
///
/// ## Intention
///
/// This type is intended to be used as a column for ECS Archetypes. In which
/// case it is not necessary to store the capacity or the size for each given
/// column. The Archetype maintains how many rows each column has and how many
/// elements they contain. It would be wasteful to store this information for
/// every column individually.
///
/// For similar reasons, the interface is very simplistic, leaving room for a
/// number of potential coding errors. The user can easily construct a [Blob]
/// which has holes of uninitialized memory in it. This can be problematic if
/// the stored type needs to be [drop]-ed.
///
/// It should be used as a regular vector where elements are pushed/popped to
/// the back, but this cannot be enforced, so in practice [Blob] can be used
/// as the user wishes.
#[derive(Debug)]
pub struct Blob {
    item_layout: core::alloc::Layout,
    stride: usize,
    data: ptr::NonNull<u8>,
}

impl Blob {
    /// Create [Blob] with given capacity
    ///
    /// It will allocate the required `uninitialized` memory block.
    /// The block will be properly aligned for each (or the only) element.
    /// `capacity` of 0 is valid, however no element may be added until it
    /// memory has been allocated.
    /// TODO: Zero sized types are supported.
    ///
    /// # Panics
    ///
    /// If too much capacity is to be allocated or the OS ran out of memory.
    /// In this case the program terminates in OS dependent manner.
    pub fn with_capacity(item_layout: alloc::Layout, capacity: usize) -> Self {
        let (data, stride) = Self::allocate_blob(item_layout, capacity);

        Self {
            item_layout,
            stride,
            data,
        }
    }

    /// Get pointer to the `index` element
    ///
    /// The returned pointer will point to memory managed by the
    /// [Blob]. Reading or writing may be safe, potentially dropping
    /// the value stored at the memory locations is not.
    /// TODO: introduce a type encapsulating this restriction
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `index` must be smaller than the allocated capacity
    /// - the returned pointer can only be read/written if it is considered
    ///   valid
    pub unsafe fn get(&self, index: usize) -> ptr::NonNull<u8> {
        unsafe { self.data.byte_add(self.stride * index) }
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }

    pub fn get_non_null_ptr(&self) -> ptr::NonNull<u8> {
        self.data
    }

    /// Copy a region of memory into [Blob]
    ///
    /// The operation does not modify `item`.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `item` does not overlap with the memory found at `index`
    /// - memory at `index` is uninitialized
    /// - `item` can be interpreted as a valid type stored in [Blob]
    /// - `item` must be at least as big as the type stored in [Blob], but it
    ///   does not need to have additional padding
    /// - index must be smaller than the allocated capacity
    pub unsafe fn set(&mut self, index: usize, item: ptr::NonNull<u8>) {
        unsafe {
            ptr::copy_nonoverlapping(
                item.as_ptr(),
                self.data.as_ptr().byte_add(self.stride * index),
                self.item_layout.size(),
            );
        }
    }

    /// Swap item sized memory between  offsets `index_a` and `index_b`
    ///
    /// `index_a` and `index_b` may be equal, in this case it is a no-op.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - both index must be less than the current capacity
    pub unsafe fn swap(&mut self, index_a: usize, index_b: usize) {
        if index_a == index_b {
            return;
        }
        unsafe {
            let a = self.data.byte_add(self.stride * index_a);
            let b = self.data.byte_add(self.stride * index_b);
            ptr::swap_nonoverlapping(a.as_ptr(), b.as_ptr(), self.item_layout.size());
        }
    }

    /// Swap item sized memory between from `index` with the last element and
    /// destroy it
    ///
    /// `index_a` and `index_b` may be equal, in this case it is a no-op.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `size` is the count of the continuous initialized memory locations
    ///   from the start of the container
    /// - `index` is smaller than `size`
    /// - `item_drop_fn` is [None], if and only if the stored type does not
    ///   implement a [drop]
    /// - `item_drop_fn` properly calls [drop] for the stored type
    pub unsafe fn swap_remove(
        &mut self,
        index: usize,
        size: usize,
        item_drop_fn: Option<ComponentDropFn>,
    ) {
        unsafe {
            self.swap(index, size - 1);
            self.drop_item(index, item_drop_fn);
        }
    }

    /// Clear the container
    ///
    /// Call [drop] on the first `size` count elements.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `size` is the count of the continuous initialized memory locations
    ///   from the start of the container
    /// - `item_drop_fn` is [None], if and only if the stored type does not
    ///   implement a [drop]
    /// - `item_drop_fn` properly calls [drop] for the stored type
    pub unsafe fn clear(&mut self, size: usize, item_drop_fn: ComponentDropFn) {
        for index in 0..size {
            unsafe {
                self.drop_item(index, Some(item_drop_fn));
            }
        }
    }

    /// Grow the [Blob]
    ///
    /// In case `capacity` is equal to `new_capacity` it is a no-op.
    /// Otherwise it will allocate `new_capacity` and memory move `size` number
    /// of elements.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `size` is the count of the continuous initialized memory locations
    ///   from the start of the container
    /// - `size` <= `capacity`
    /// - `new_capacity` >= `capacity`
    ///
    /// # Panics
    ///
    /// If too much capacity is to be allocated or the OS ran out of memory.
    /// In this case the program terminates in OS dependent manner.
    pub unsafe fn grow(&mut self, size: usize, capacity: usize, new_capacity: usize) {
        debug_assert!(size <= capacity, "Size cannot be greater than capacity.");
        debug_assert!(new_capacity >= capacity, "Grow cannot shrink");
        if capacity == new_capacity {
            return;
        }
        let (mut data, _stride) = Self::allocate_blob(self.item_layout, new_capacity);

        if self.item_layout.size() == 0 || capacity == 0 {
            self.data = data;
        } else {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    data.as_ptr(),
                    self.stride * size,
                );
                mem::swap(&mut self.data, &mut data);
                let (old_array_layout, _stride) = self.item_layout.repeat(capacity).unwrap();
                alloc::dealloc(data.as_ptr(), old_array_layout);
            }
        }
    }

    /// Shrink the [Blob]
    ///
    /// In case `capacity` is equal to `new_capacity` it is a no-op.
    /// Otherwise it will allocate `new_capacity` and memory move `size` number
    /// of elements. If `new_capacity` is smaller than `size` then only
    /// `new_capacity` elements will be kept, the rest will be released by
    /// the `item_drop_fn`.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `size` is the count of the continuous initialized memory locations
    ///   from the start of the container
    /// - `size` <= `capacity`
    /// - `new_capacity`<= `capacity`
    /// - `item_drop_fn` is [None], if and only if the stored type does not
    ///   implement a [drop]
    /// - `item_drop_fn` properly calls [drop] for the stored type
    pub unsafe fn shrink(
        &mut self,
        size: usize,
        capacity: usize,
        new_capacity: usize,
        item_drop_fn: Option<ComponentDropFn>,
    ) {
        debug_assert!(size <= capacity, "Size cannot be greater than capacity.");
        debug_assert!(new_capacity <= capacity, "Shrink cannot grow");
        if capacity == new_capacity {
            return;
        }
        let (mut data, _stride) = Self::allocate_blob(self.item_layout, new_capacity);

        // drop elements that need to be dropped
        if new_capacity < size {
            for index in new_capacity..size {
                unsafe {
                    self.drop_item(index, item_drop_fn);
                }
            }
        }

        let to_copy_size = cmp::min(size, new_capacity);
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.data.as_ptr(),
                data.as_ptr(),
                self.stride * to_copy_size,
            );

            mem::swap(&mut self.data, &mut data);

            let (old_array_layout, _stride) = self.item_layout.repeat(capacity).unwrap();
            alloc::dealloc(data.as_ptr(), old_array_layout);
        }
    }

    /// Resize the [Blob]
    ///
    /// Either [shrink](Blob::shrink) or [grow](Blob::grow) the [Blob] based on
    /// the `capacity` and `new_capacity`.
    ///
    /// ## Safety
    ///
    /// The restrictions from [shrink](Blob::shrink) and [grow](Blob::grow)
    /// apply.
    ///
    /// # Panics
    ///
    /// If too much capacity is to be allocated or the OS ran out of memory.
    /// In this case the program terminates in OS dependent manner.
    pub unsafe fn resize(
        &mut self,
        size: usize,
        capacity: usize,
        new_capacity: usize,
        item_drop_fn: Option<ComponentDropFn>,
    ) {
        if capacity <= new_capacity {
            unsafe {
                self.shrink(size, capacity, new_capacity, item_drop_fn);
            }
        } else {
            unsafe {
                self.grow(size, capacity, new_capacity);
            }
        }
    }

    /// Release resources
    ///
    /// It will call the supplied [drop] function on all the elements until
    /// index `size`. Then it will release the memory for the array itself.
    ///
    /// This function must be called to avoid memory leaks! The [Blob] itself
    /// cannot implement it's own drop function, as it does not know of the
    /// stored types.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `size` is the count of the continuous initialized memory locations
    ///   from the start of the container
    /// - `size` <= `capacity`
    /// - `item_drop_fn` is [None], if and only if the stored type does not
    ///   implement a [drop]
    /// - `item_drop_fn` properly calls [drop] for the stored type
    pub unsafe fn drop(
        &mut self,
        size: usize,
        capacity: usize,
        item_drop_fn: Option<ComponentDropFn>,
    ) {
        if self.item_layout.size() == 0 || capacity == 0 {
            return;
        }

        if let Some(drop_fn) = item_drop_fn {
            unsafe {
                self.clear(size, drop_fn);
            }
        }

        let (array_layout, _) = self.item_layout.repeat(capacity).unwrap();
        unsafe {
            alloc::dealloc(self.data.as_ptr().cast(), array_layout);
        }
    }

    fn allocate_blob(item_layout: alloc::Layout, capacity: usize) -> (ptr::NonNull<u8>, usize) {
        // In case of ZST of 0 capacity there is nothing to allocate and the global
        // allocator may not be called with 0, but the pointer must be properly aligned
        // and the stride calculated appropriately.
        let (data, stride) = if item_layout.size() == 0 || capacity == 0 {
            let alignment = num::NonZeroUsize::new(item_layout.align()).unwrap();
            // By offsetting the null pointer with the alignment, the
            // resulting address is properly aligned.
            // Passing it to without_provenance creates a pointer which
            // is properly aligned, but cannot be accessed neither read or write.
            let data = ptr::NonNull::<u8>::without_provenance(alignment);
            // It is crucial we calculate the stride here properly. It is not the
            // alignment nor the the size of the type, but their least common multiple.
            (data, item_layout.pad_to_align().size())
        } else {
            // Layout.repeat fails on arithmetic overflow. Meaning that more memory
            // allocation was attempted than can possibly be addressed.
            let (array_layout, stride) = item_layout.repeat(capacity).unwrap();
            let data_allocation = unsafe { alloc::alloc(array_layout) };
            // If we couldn't allocate the required memory we can just panic.
            // There is absolutely no reason to return a Result as the user simply
            // cannot do anything about it.
            if data_allocation.is_null() {
                alloc::handle_alloc_error(array_layout);
            }

            (
                unsafe { ptr::NonNull::new_unchecked(data_allocation) },
                stride,
            )
        };

        (data, stride)
    }

    /// Directly call `item_drop_fn` on the `indexed` item
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `size` is the count of the continuous initialized memory locations
    ///   from the start of the container
    /// - `index` <= `size`
    /// - `new_capacity`<= `capacity`
    /// - `item_drop_fn` is [None], if and only if the stored type does not
    ///   implement a [drop]
    /// - `item_drop_fn` properly calls [drop] for the stored type
    unsafe fn drop_item(&self, index: usize, item_drop_fn: Option<ComponentDropFn>) {
        if let Some(drop_fn) = item_drop_fn {
            unsafe {
                drop_fn(self.get(index));
            }
        }
    }

    /// Get a `&T` iterator to [Blob]
    ///
    /// Can't provide an [IntoIterator] implementation directly as [Blob] does
    /// not know the number of elements, the size of the storage or the
    /// stored type.
    ///
    /// ## Safety
    ///
    /// Behaviour is undefined if any of the following are violated:
    ///
    /// - `len` does not match the number of `stored` elements
    /// - [Blob] must be managing memory for types `T`
    pub unsafe fn iter<'a, T>(&'a self, len: usize) -> BlobIterator<'a, T> {
        BlobIterator {
            index: 0,
            len,
            blob: self,
            phantom: PhantomData,
        }
    }
}

pub struct BlobIterator<'a, T> {
    index: usize,
    len: usize,
    blob: &'a Blob,
    phantom: PhantomData<&'a T>,
}

// impl<'a, T> BlobIterator<'a, T> {
//     pub fn empty() -> BlobIterator<'a, T> {
//         BlobIterator {
//             index: 0,
//             len: 0,
//             blob: &'a Blob::with_capacity(alloc::Layout::new::<T>(), 0),
//             phantom: PhantomData,
//         }
//     }
// }

impl<'a, T> Iterator for BlobIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = if self.index < self.len {
            Some(unsafe { self.blob.get(self.index).cast::<T>().as_ref() })
        } else {
            None
        };
        self.index += 1;

        item
    }
}

#[cfg(test)]
mod no_drop_needed {
    use std::{alloc, ptr};

    use crate::blob::Blob;

    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    struct SomeData {
        a: i64,
        b: u8,
        c: u32,
        d: u8,
    }
    const TEST_DATA: [SomeData; 3] = [
        SomeData {
            a: -135,
            b: 0,
            c: 44444,
            d: 50,
        },
        SomeData {
            a: -75,
            b: 1,
            c: 33333,
            d: 100,
        },
        SomeData {
            a: -50,
            b: 2,
            c: 22222,
            d: 150,
        },
    ];

    fn init_3_elements(blob: &mut Blob) {
        let mut element_0 = Box::new(TEST_DATA[0]);
        unsafe {
            blob.set(
                0,
                ptr::NonNull::new((element_0.as_mut() as *mut SomeData).cast::<u8>()).unwrap(),
            );
        }

        let mut element_1 = Box::new(TEST_DATA[1]);
        unsafe {
            blob.set(
                1,
                ptr::NonNull::new((element_1.as_mut() as *mut SomeData).cast::<u8>()).unwrap(),
            );
        }

        let mut element_2 = Box::new(TEST_DATA[2]);
        unsafe {
            blob.set(
                2,
                ptr::NonNull::new((element_2.as_mut() as *mut SomeData).cast::<u8>()).unwrap(),
            );
        }
    }

    #[test]
    fn init_and_drop_empty() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 0);

        unsafe {
            blob.drop(0, 0, None);
        }
    }

    #[test]
    fn write_and_read() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 3);
        init_3_elements(&mut blob);
        {
            let read_0 = unsafe { blob.get(0).cast::<SomeData>().as_ref() };
            let read_1 = unsafe { blob.get(1).cast::<SomeData>().as_ref() };
            let read_2 = unsafe { blob.get(2).cast::<SomeData>().as_ref() };

            assert_eq!(*read_0, TEST_DATA[0]);
            assert_eq!(*read_1, TEST_DATA[1]);
            assert_eq!(*read_2, TEST_DATA[2]);
        }

        unsafe {
            blob.drop(3, 3, None);
        }
    }

    #[test]
    fn swap_around() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 3);
        init_3_elements(&mut blob);

        unsafe {
            blob.swap(0, 2);
            blob.swap(1, 2);

            let read_0 = blob.get(0).cast::<SomeData>().as_ref();
            let read_1 = blob.get(1).cast::<SomeData>().as_ref();
            let read_2 = blob.get(2).cast::<SomeData>().as_ref();

            assert_eq!(*read_0, TEST_DATA[2]);
            assert_eq!(*read_1, TEST_DATA[0]);
            assert_eq!(*read_2, TEST_DATA[1]);
        }

        unsafe {
            blob.drop(3, 3, None);
        }
    }

    #[test]
    fn swap_remove() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 3);
        init_3_elements(&mut blob);

        unsafe {
            blob.swap_remove(1, 3, None);
            let read_0 = blob.get(0).cast::<SomeData>().as_ref();
            let read_1 = blob.get(1).cast::<SomeData>().as_ref();

            assert_eq!(*read_0, TEST_DATA[0]);
            assert_eq!(*read_1, TEST_DATA[2]);
        }

        unsafe {
            blob.swap_remove(0, 2, None);
            let read_0 = blob.get(0).cast::<SomeData>().as_ref();

            assert_eq!(*read_0, TEST_DATA[2]);
            blob.swap_remove(0, 1, None);
        }

        unsafe {
            blob.drop(0, 3, None);
        }
    }

    #[test]
    fn grow() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 0);

        unsafe {
            blob.grow(0, 0, 3);
        }
        init_3_elements(&mut blob);

        {
            let read_0 = unsafe { blob.get(0).cast::<SomeData>().as_ref() };
            let read_1 = unsafe { blob.get(1).cast::<SomeData>().as_ref() };
            let read_2 = unsafe { blob.get(2).cast::<SomeData>().as_ref() };

            assert_eq!(*read_0, TEST_DATA[0]);
            assert_eq!(*read_1, TEST_DATA[1]);
            assert_eq!(*read_2, TEST_DATA[2]);
        }

        unsafe {
            blob.grow(3, 3, 5);
        }

        let data_3 = SomeData {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        };
        let mut element_3 = Box::new(data_3);

        unsafe {
            blob.set(
                3,
                ptr::NonNull::new((element_3.as_mut() as *mut SomeData).cast::<u8>()).unwrap(),
            );
        }

        let data_4 = SomeData {
            a: 9,
            b: 9,
            c: 9,
            d: 9,
        };
        let mut element_4 = Box::new(data_4);

        unsafe {
            blob.set(
                4,
                ptr::NonNull::new((element_4.as_mut() as *mut SomeData).cast::<u8>()).unwrap(),
            );
            // Repeated grow should have no effect
            blob.grow(5, 5, 5);
        }

        {
            let read_0 = unsafe { blob.get(0).cast::<SomeData>().as_ref() };
            let read_1 = unsafe { blob.get(1).cast::<SomeData>().as_ref() };
            let read_2 = unsafe { blob.get(2).cast::<SomeData>().as_ref() };
            let read_3 = unsafe { blob.get(3).cast::<SomeData>().as_ref() };
            let read_4 = unsafe { blob.get(4).cast::<SomeData>().as_ref() };

            assert_eq!(*read_0, TEST_DATA[0]);
            assert_eq!(*read_1, TEST_DATA[1]);
            assert_eq!(*read_2, TEST_DATA[2]);
            assert_eq!(*read_3, data_3);
            assert_eq!(*read_4, data_4);
        }

        unsafe {
            blob.drop(5, 5, None);
        }
    }

    #[test]
    fn shrink() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 5);
        init_3_elements(&mut blob);

        unsafe {
            blob.shrink(3, 5, 3, None);

            let read_0 = blob.get(0).cast::<SomeData>().as_ref();
            let read_1 = blob.get(1).cast::<SomeData>().as_ref();
            let read_2 = blob.get(2).cast::<SomeData>().as_ref();

            assert_eq!(*read_0, TEST_DATA[0]);
            assert_eq!(*read_1, TEST_DATA[1]);
            assert_eq!(*read_2, TEST_DATA[2]);
        }

        unsafe {
            blob.shrink(3, 3, 1, None);

            let read_0 = blob.get(0).cast::<SomeData>().as_ref();

            assert_eq!(*read_0, TEST_DATA[0]);
        }

        unsafe {
            blob.shrink(1, 1, 0, None);
            // Shrink without size change should have no effect
            blob.shrink(0, 0, 0, None);
        }
    }

    #[test]
    fn iter_ref() {
        let layout = alloc::Layout::new::<SomeData>();
        let mut blob = Blob::with_capacity(layout, 5);
        init_3_elements(&mut blob);

        let mut iter = unsafe { blob.iter::<SomeData>(3) };

        let read_0 = iter.next().unwrap();
        let read_1 = iter.next().unwrap();
        let read_2 = iter.next().unwrap();
        assert_eq!(iter.next(), None);

        assert_eq!(*read_0, TEST_DATA[0]);
        assert_eq!(*read_1, TEST_DATA[1]);
        assert_eq!(*read_2, TEST_DATA[2]);

        // Do it a second time to make sure the no modifications occurred.
        let mut iter = unsafe { blob.iter::<SomeData>(3) };

        let read_0 = iter.next().unwrap();
        let read_1 = iter.next().unwrap();
        let read_2 = iter.next().unwrap();
        assert_eq!(iter.next(), None);

        assert_eq!(*read_0, TEST_DATA[0]);
        assert_eq!(*read_1, TEST_DATA[1]);
        assert_eq!(*read_2, TEST_DATA[2]);

        unsafe {
            blob.drop(3, 5, None);
        }
    }
}

// TODO tests with types requiring drop
// TODO tests with ZST types
