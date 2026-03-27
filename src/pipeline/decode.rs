use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("JPEG decode failed: {0}")]
    DecodeFailed(String),
    #[error("Buffer pool exhausted")]
    PoolExhausted,
}

/// RGBA image buffer
pub struct ImageBuffer {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
}

impl ImageBuffer {
    pub fn new(width: u32, height: u32, channels: u32) -> Self {
        let size = (width * height * channels) as usize;
        Self {
            data: vec![0u8; size],
            width,
            height,
            channels,
        }
    }

    pub fn from_jpeg(data: &[u8]) -> Result<Self, DecodeError> {
        // Use image crate to decode JPEG
        let img =
            image::load_from_memory(data).map_err(|e| DecodeError::DecodeFailed(e.to_string()))?;

        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let pixels = rgba_img.into_raw();

        Ok(Self {
            data: pixels,
            width,
            height,
            channels: 4,
        })
    }
}

/// Buffer pool for pre-allocated JPEG decode buffers
pub struct BufferPool {
    buffers: Arc<Mutex<Vec<ImageBuffer>>>,
    capacity: usize,
}

impl BufferPool {
    pub fn new(capacity: usize, _buffer_size: u32) -> Self {
        let mut buffers = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffers.push(ImageBuffer::new(256, 256, 4)); // 256x256 RGBA
        }
        Self {
            buffers: Arc::new(Mutex::new(buffers)),
            capacity,
        }
    }

    pub fn acquire(&self) -> Result<ImageBuffer, DecodeError> {
        let mut pool = self.buffers.lock().expect("buffer pool mutex poisoned");
        pool.pop().ok_or(DecodeError::PoolExhausted)
    }

    pub fn release(&self, buffer: ImageBuffer) {
        let mut pool = self.buffers.lock().expect("buffer pool mutex poisoned");
        if pool.len() < self.capacity {
            pool.push(buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_buffer_creation() {
        let buf = ImageBuffer::new(256, 256, 4);
        assert_eq!(buf.width, 256);
        assert_eq!(buf.height, 256);
        assert_eq!(buf.channels, 4);
        assert_eq!(buf.data.len(), 256 * 256 * 4);
    }

    #[test]
    fn test_buffer_pool_acquire_release() {
        let pool = BufferPool::new(2, 256);

        // Acquire two buffers
        let buf1 = pool.acquire().unwrap();
        let _buf2 = pool.acquire().unwrap();

        // Third acquire should fail
        assert!(pool.acquire().is_err());

        // Release one
        pool.release(buf1);

        // Now we can acquire again
        assert!(pool.acquire().is_ok());
    }

    #[test]
    fn test_buffer_pool_capacity() {
        let pool = BufferPool::new(3, 256);

        let _b1 = pool.acquire().unwrap();
        let _b2 = pool.acquire().unwrap();
        let _b3 = pool.acquire().unwrap();

        assert!(pool.acquire().is_err());
    }
}
