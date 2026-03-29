use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("JPEG decode failed: {0}")]
    DecodeFailed(String),
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
}
