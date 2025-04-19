use std::{
    ops::{Range, RangeBounds},
    usize,
};

use wgpu::{BufferSize, BufferSlice, Queue, QueueWriteBufferView};

#[derive(Debug)]
pub struct SlicedBuffer {
    pub buffer: wgpu::Buffer,
    slices: Vec<Range<usize>>,
    capacity: wgpu::BufferAddress,
}

impl SlicedBuffer {
    pub fn new(buffer: wgpu::Buffer, capacity: wgpu::BufferAddress) -> Self {
        Self {
            buffer,
            slices: Vec::with_capacity(64),
            capacity,
        }
    }

    pub fn get_slice(&self, range: impl RangeBounds<u64>) -> BufferSlice<'_> {
        self.buffer.slice(range)
    }

    pub fn slices(&self) -> &[Range<usize>] {
        &self.slices
    }

    pub fn write_into<'a>(
        &'a self,
        queue: &'a Queue,
        size: BufferSize,
    ) -> QueueWriteBufferView<'a> {
        queue
            .write_buffer_with(&self.buffer, 0, size)
            .expect("Failed to create staging buffer for vertex data")
    }
}
