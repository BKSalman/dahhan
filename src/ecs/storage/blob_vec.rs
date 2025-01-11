use std::{alloc::Layout, mem::ManuallyDrop};

#[derive(Debug)]
pub struct BlobVec {
    item_layout: Layout,
    data: ManuallyDrop<Vec<u8>>,
    drop_fn: fn(*mut ()),
}

#[cfg(test)]
impl PartialEq for BlobVec {
    fn eq(&self, other: &Self) -> bool {
        self.item_layout == other.item_layout && self.data == other.data
    }
}

impl BlobVec {
    pub fn new<T>() -> Self {
        Self {
            item_layout: Layout::new::<T>(),
            data: unsafe { ManuallyDrop::new(std::mem::transmute(Vec::<T>::new())) },
            drop_fn: unsafe {
                std::mem::transmute(std::ptr::drop_in_place::<Vec<T>> as unsafe fn(*mut Vec<T>))
            },
        }
    }

    unsafe fn typed_ref<T>(&self) -> &Vec<T> {
        std::mem::transmute(&self.data)
    }

    unsafe fn typed_mut<T>(&mut self) -> &mut Vec<T> {
        std::mem::transmute(&mut self.data)
    }

    /// Pushes a new element of type `T` into the vector
    ///
    /// # Panics
    /// Panics if the item being pushed doesn't match the layout of the vector items
    ///
    /// # Safety
    /// The pushed item type MUST have a layout that matches the items in the vector
    pub unsafe fn push<T>(&mut self, item: T) {
        assert!(Layout::new::<T>() == self.item_layout);

        self.typed_mut().push(item);
    }

    /// Returns a reference to the element at the given index
    ///
    /// # Panics
    /// Panics if the item being pushed doesn't match the layout of the vector items
    ///
    /// # Safety
    /// The requested item type MUST have a layout that matches the items in the vector
    pub unsafe fn get<T>(&self, index: usize) -> Option<&T> {
        assert!(Layout::new::<T>() == self.item_layout);

        self.typed_ref().get(index)
    }

    /// Returns a mutable reference to the element at the given index
    ///
    /// # Panics
    /// Panics if the item being pushed doesn't match the layout of the vector items
    ///
    /// # Safety
    /// The requested item type MUST have a layout that matches the items in the vector
    pub unsafe fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        assert!(Layout::new::<T>() == self.item_layout);

        self.typed_mut().get_mut(index)
    }

    // TODO: handle 1 element
    pub unsafe fn swap_remove(&mut self, index: usize) {
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }

        let len = self.data.len();
        if len == 1 {
            self.data.set_len(0);
            return;
        }

        let bytes_len = len * self.item_layout.size();
        eprintln!("len: {bytes_len}");
        if index >= len {
            assert_failed(index, len);
        }
        let base_ptr = self.data.as_mut_ptr();
        std::ptr::swap_nonoverlapping(
            base_ptr.add(bytes_len - self.item_layout.size()),
            base_ptr.add(index * self.item_layout.size()),
            self.item_layout.size(),
        );
        self.data.set_len(len - 1);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        let v: &mut Vec<u8> = &mut self.data;
        let v: *mut Vec<u8> = v as *mut _;
        let v: *mut () = v as _;
        (self.drop_fn)(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct SomeComponent {
        something: u32,
    }

    #[test]
    fn test_blob_vec() {
        let mut vec = BlobVec::new::<SomeComponent>();

        unsafe { vec.push(SomeComponent { something: 10 }) };

        eprintln!("{vec:?}");

        let component = unsafe { vec.get::<SomeComponent>(0) };

        assert_eq!(component, Some(&SomeComponent { something: 10 }));

        unsafe { vec.push(SomeComponent { something: 5 }) };

        eprintln!("{vec:?}");

        let component = unsafe { vec.get::<SomeComponent>(1) };

        assert_eq!(component, Some(&SomeComponent { something: 5 }));
    }

    #[test]
    fn test_swap_remove() {
        let mut vec = BlobVec::new::<SomeComponent>();

        unsafe { vec.push(SomeComponent { something: 1 }) };
        unsafe { vec.push(SomeComponent { something: 2 }) };
        unsafe { vec.push(SomeComponent { something: 3 }) };

        unsafe { vec.swap_remove(0) };

        let mut expected = BlobVec::new::<SomeComponent>();
        unsafe { expected.push(SomeComponent { something: 3 }) };
        unsafe { expected.push(SomeComponent { something: 2 }) };

        assert_eq!(vec, expected);
    }

    #[test]
    fn test_swap_remove_single_element() {
        let mut vec = BlobVec::new::<SomeComponent>();

        unsafe { vec.push(SomeComponent { something: 1 }) };

        unsafe { vec.swap_remove(0) };

        let expected = BlobVec::new::<SomeComponent>();

        assert_eq!(vec, expected);
    }
}
