use std::{cmp, mem, ptr};
use std::num::UnsignedInt;
use std::old_io::IoResult;
use std::raw;
use alloc::heap;
use super::{Buf, MutBuf};

pub struct ByteBuf {
    ptr: *mut u8,
    cap: usize,
    pos: usize,
    lim: usize
}

impl ByteBuf {
    pub fn new(mut capacity: usize) -> ByteBuf {
        // Handle 0 capacity case
        if capacity == 0 {
            return ByteBuf {
                ptr: ptr::null_mut(),
                cap: 0,
                pos: 0,
                lim: 0
            }
        }

        capacity = UnsignedInt::next_power_of_two(capacity);

        let ptr = unsafe { heap::allocate(capacity, mem::min_align_of::<u8>()) };

        ByteBuf {
            ptr: ptr as *mut u8,
            cap: capacity,
            pos: 0,
            lim: capacity
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn flip(&mut self) {
        self.lim = self.pos;
        self.pos = 0;
    }

    pub fn clear(&mut self) {
        self.pos = 0;
        self.lim = self.cap;
    }

    fn as_ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }

    fn as_slice<'a>(&'a self) -> &'a [u8] {
        unsafe {
            mem::transmute(raw::Slice {
                data: self.as_ptr(), len: self.cap
            })
        }
    }

    fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        unsafe { mem::transmute(self.as_slice()) }
    }
}

impl Drop for ByteBuf {
    fn drop(&mut self) {
        if self.cap > 0 {
            unsafe {
                heap::deallocate(self.ptr, self.cap, mem::min_align_of::<u8>())
            }
        }
    }
}

impl Buf for ByteBuf {
    fn remaining(&self) -> usize {
        self.lim - self.pos
    }

    fn bytes<'a>(&'a self) -> &'a [u8] {
        self.as_slice().slice(self.pos, self.lim)
    }

    fn advance(&mut self, mut cnt: usize) {
        cnt = cmp::min(cnt, self.remaining());
        self.pos += cnt;
    }
}

impl MutBuf for ByteBuf {
    fn mut_bytes<'a>(&'a mut self) -> &'a mut [u8] {
        let pos = self.pos;
        let lim = self.lim;
        self.as_mut_slice().slice_mut(pos, lim)
    }
}

impl Reader for ByteBuf {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        super::read(self, buf)
    }
}

impl Writer for ByteBuf {
    fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        super::write(self, buf)
    }
}

unsafe impl Send for ByteBuf { }

#[cfg(test)]
mod test {
    use buf::*;

    #[test]
    pub fn test_initial_buf_empty() {
        let mut buf = ByteBuf::new(100);

        assert!(buf.capacity() == 128);
        assert!(buf.remaining() == 128);

        buf.flip();

        assert!(buf.remaining() == 0);

        buf.clear();

        assert!(buf.remaining() == 128);
    }

    #[test]
    pub fn test_writing_bytes() {
        let mut buf = ByteBuf::new(8);

        buf.write(b"hello").unwrap();
        assert!(buf.remaining() == 3);

        buf.flip();

        assert!(buf.read_to_end().unwrap().as_slice() == b"hello");
    }
}
