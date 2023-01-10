use std::alloc;
use std::ptr::NonNull;

pub struct MyVec<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T> MyVec<T> {
    fn new() -> Self {
        Self {
            capacity: 0,
            len: 0,
            ptr: NonNull::dangling(),
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        Some(unsafe { &*self.ptr.as_ptr().add(index) })
    }

    fn push(&mut self, item: T) {
        if std::mem::size_of::<T>() == 0 {
            panic!("No Zero Size data");
        }
        if self.capacity == 0 {
            let layout = alloc::Layout::array::<T>(4).expect("Could not allocate more elements");
            let ptr = unsafe { alloc::alloc(layout) } as *mut T;
            let ptr = NonNull::new(ptr).expect("Could not allocate");
            // SAFETY: ptr is not null and we have allocated enough space for this pointer
            unsafe { ptr.as_ptr().write(item) }
            self.ptr = ptr;
            self.capacity = 4;
            self.len = 1;
        } else if self.len < self.capacity {
            let offset = self
                .len
                .checked_mul(std::mem::size_of::<T>())
                .expect("Cannot reach to memory location");
            assert!(offset < isize::MAX as usize, "Wrapped Isize");
            // offset cannot wrap around
            // pointer is pointing to valid memory
            unsafe { self.ptr.as_ptr().add(self.len).write(item) }
            self.len += 1;
        } else {
            let new_capacity = self.capacity.checked_mul(2).expect("Capacity reached");
            // need to look at this code again from library perspective
            // what is align?

            let align = std::mem::align_of::<T>();
            let size = std::mem::size_of::<T>() * self.capacity;
            let rounded_size = size.checked_add(size % align).expect("cannot do modulo");
            unsafe {
                let layout = alloc::Layout::from_size_align_unchecked(rounded_size, align);
                let new_size = std::mem::size_of::<T>() * new_capacity;
                let ptr = alloc::realloc(self.ptr.as_ptr() as *mut u8, layout, new_size) as *mut T;
                let ptr = NonNull::new(ptr).expect("Could not allocate");
                // SAFETY: ptr is not null and we have allocated enough space for this pointer
                ptr.as_ptr().add(self.len).write(item);
                self.ptr = ptr;
                self.capacity = new_capacity;
                self.len += 1
            }
        }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len));
            let layout = alloc::Layout::from_size_align_unchecked(
                std::mem::size_of::<T>() * self.capacity,
                std::mem::align_of::<T>(),
            );
            alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut vec = MyVec::<usize>::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);
        assert_eq!(vec.get(3), Some(&4));
        assert_eq!(vec.capacity(), 8);
        assert_eq!(vec.len(), 5);
    }
}
