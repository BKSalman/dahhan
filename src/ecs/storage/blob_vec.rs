use std::alloc::Layout;

#[derive(Debug, PartialEq)]
pub struct BlobVec {
    item_layout: Layout,
    data: Vec<u8>,
    drop_fn: Option<fn(*mut u8)>,
}

impl BlobVec {
    pub fn new<T>() -> Self {
        let layout = Layout::new::<T>();

        Self {
            item_layout: layout,
            data: Vec::new(),
            drop_fn: if std::mem::needs_drop::<T>() {
                Some(|ptr| unsafe {
                    std::ptr::drop_in_place(ptr.cast::<T>());
                })
            } else {
                None
            },
        }
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

        let required_bytes = self.data.len() + self.item_layout.size();
        if required_bytes > self.data.capacity() {
            self.data.reserve(self.item_layout.size());
        }

        let start = self.data.len();
        self.data.set_len(start + self.item_layout.size());
        std::ptr::write(self.data.as_mut_ptr().add(start).cast(), item);
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

        let offset = index.checked_mul(self.item_layout.size())?;
        if offset < self.data.len() {
            Some(&*(self.data.as_ptr().add(offset).cast()))
        } else {
            None
        }
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

        let offset = index.checked_mul(self.item_layout.size())?;
        if offset < self.data.len() {
            Some(&mut *(self.data.as_mut_ptr().add(offset).cast()))
        } else {
            None
        }
    }

    ///
    /// # Safety
    /// The requested item type MUST have a layout that matches the items in the vector
    pub unsafe fn swap_remove<T>(&mut self, index: usize) -> T {
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }

        let len = self.data.len();
        if index >= len {
            assert_failed(index, len);
        }

        let value = std::ptr::read(self.data.as_ptr().add(index).cast::<T>());
        let base_ptr = self.data.as_mut_ptr();
        std::ptr::copy(
            base_ptr.add(len - self.item_layout.size()),
            base_ptr.add(index),
            self.item_layout.size(),
        );
        self.data.set_len(len - self.item_layout.size());

        value
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.item_layout.size()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity() / self.item_layout.size()
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            let item_size = self.item_layout.size();
            let len = self.len();
            for i in 0..len {
                let offset = i * item_size;
                unsafe {
                    drop_fn(self.data.as_mut_ptr().add(offset));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct SomeComponent {
        something: u32,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct OtherComponent;

    #[test]
    fn test_blob_vec() {
        let mut vec = BlobVec::new::<SomeComponent>();

        unsafe { vec.push(SomeComponent { something: 10 }) };

        eprintln!("{vec:?}");

        let component = unsafe { vec.get::<SomeComponent>(0) };

        assert_eq!(component, Some(&SomeComponent { something: 10 }));
    }

    #[test]
    fn test_swap_remove() {
        let mut vec = BlobVec::new::<SomeComponent>();

        unsafe { vec.push(SomeComponent { something: 1 }) };
        unsafe { vec.push(SomeComponent { something: 2 }) };
        unsafe { vec.push(SomeComponent { something: 3 }) };

        eprintln!("{vec:?}");

        let component = unsafe { vec.swap_remove::<SomeComponent>(0) };

        assert_eq!(component, SomeComponent { something: 1 });

        let mut expected = BlobVec::new::<SomeComponent>();
        unsafe { expected.push(SomeComponent { something: 3 }) };
        unsafe { expected.push(SomeComponent { something: 2 }) };

        assert_eq!(vec, expected);
    }
}
