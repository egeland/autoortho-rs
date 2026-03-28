use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Invalid bounds")]
    InvalidBounds,
    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// RGBA image buffer (4 channels, row-major, no padding)
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Image {
    /// Create a new zero-filled image
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            data: vec![0u8; size],
        }
    }

    /// Create new image filled with a color
    pub fn new_filled(width: u32, height: u32, rgba: [u8; 4]) -> Self {
        let pixel_count = (width as usize) * (height as usize);
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.extend_from_slice(&rgba);
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Create image from existing RGBA data
    pub fn from_raw(width: u32, height: u32, data: Vec<u8>) -> Result<Self, ImageError> {
        let expected = (width as usize) * (height as usize) * 4;
        if data.len() != expected {
            return Err(ImageError::InvalidDimensions(format!(
                "expected {} bytes for {}x{} RGBA, got {}",
                expected,
                width,
                height,
                data.len()
            )));
        }
        Ok(Self {
            width,
            height,
            data,
        })
    }

    /// Stride in bytes (width * 4 for RGBA with no padding)
    pub fn stride(&self) -> usize {
        self.width as usize * 4
    }

    /// Paste another image onto this one at given offset
    pub fn paste(&mut self, x: u32, y: u32, src: &Image) -> Result<(), ImageError> {
        if x + src.width > self.width || y + src.height > self.height {
            return Err(ImageError::InvalidBounds);
        }

        let src_stride = src.stride();
        let dst_stride = self.stride();

        for dy in 0..src.height as usize {
            let src_start = dy * src_stride;
            let src_end = src_start + src_stride;
            let dst_start = ((y as usize + dy) * dst_stride) + (x as usize * 4);
            let dst_end = dst_start + src_stride;

            self.data[dst_start..dst_end].copy_from_slice(&src.data[src_start..src_end]);
        }

        Ok(())
    }

    /// Get pixel at x, y
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y as usize) * self.stride()) + (x as usize) * 4;
        Some([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
    }

    /// Set pixel at x, y
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) -> Result<(), ImageError> {
        if x >= self.width || y >= self.height {
            return Err(ImageError::InvalidBounds);
        }
        let idx = ((y as usize) * self.stride()) + (x as usize) * 4;
        self.data[idx..idx + 4].copy_from_slice(&rgba);
        Ok(())
    }

    /// Reduce image by half using 2x2 box filter (for mipmap generation).
    /// Each output pixel is the average of a 2x2 block of input pixels.
    /// Matches the original C `aoimage_reduce_2` / `aodds_reduce_half`.
    pub fn reduce_half(&self) -> Image {
        let new_w = (self.width / 2).max(1);
        let new_h = (self.height / 2).max(1);
        let src_stride = self.stride();
        let mut out = Image::new(new_w, new_h);

        for y in 0..new_h as usize {
            for x in 0..new_w as usize {
                let sx = x * 2;
                let sy = y * 2;

                // Gather 2x2 block (clamped for odd dimensions)
                let sx1 = sx.min(self.width as usize - 1);
                let sy1 = sy.min(self.height as usize - 1);
                let sx2 = (sx + 1).min(self.width as usize - 1);
                let sy2 = (sy + 1).min(self.height as usize - 1);

                let i00 = sy1 * src_stride + sx1 * 4;
                let i10 = sy1 * src_stride + sx2 * 4;
                let i01 = sy2 * src_stride + sx1 * 4;
                let i11 = sy2 * src_stride + sx2 * 4;

                let dst_idx = y * (new_w as usize * 4) + x * 4;
                for c in 0..4 {
                    let sum = self.data[i00 + c] as u32
                        + self.data[i10 + c] as u32
                        + self.data[i01 + c] as u32
                        + self.data[i11 + c] as u32;
                    out.data[dst_idx + c] = (sum / 4) as u8;
                }
            }
        }

        out
    }

    /// Repeatedly halve the image `steps` times.
    pub fn reduce_n(&self, steps: u32) -> Image {
        let mut current = self.reduce_half();
        for _ in 1..steps {
            current = current.reduce_half();
        }
        current
    }

    /// Crop a rectangular region from this image.
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Image, ImageError> {
        if x + w > self.width || y + h > self.height {
            return Err(ImageError::InvalidBounds);
        }

        let src_stride = self.stride();
        let mut out = Image::new(w, h);
        let dst_stride = out.stride();

        for dy in 0..h as usize {
            let src_start = (y as usize + dy) * src_stride + x as usize * 4;
            let dst_start = dy * dst_stride;
            out.data[dst_start..dst_start + dst_stride]
                .copy_from_slice(&self.data[src_start..src_start + dst_stride]);
        }

        Ok(out)
    }

    /// Upscale image by an integer factor using nearest-neighbor sampling.
    /// Used for fallback resolution: a 64×64 crop upscaled 4× → 256×256.
    pub fn upscale(&self, factor: u32) -> Image {
        let new_w = self.width * factor;
        let new_h = self.height * factor;
        let mut out = Image::new(new_w, new_h);
        let src_stride = self.stride();

        for y in 0..new_h as usize {
            let sy = y / factor as usize;
            for x in 0..new_w as usize {
                let sx = x / factor as usize;
                let src_idx = sy * src_stride + sx * 4;
                let dst_idx = y * (new_w as usize * 4) + x * 4;
                out.data[dst_idx..dst_idx + 4].copy_from_slice(&self.data[src_idx..src_idx + 4]);
            }
        }

        out
    }

    /// Crop a region and upscale it in one step (matches `aoimage_crop_and_upscale`).
    pub fn crop_and_upscale(
        &self,
        x: u32,
        y: u32,
        crop_w: u32,
        crop_h: u32,
        scale_factor: u32,
    ) -> Result<Image, ImageError> {
        let cropped = self.crop(x, y, crop_w, crop_h)?;
        Ok(cropped.upscale(scale_factor))
    }

    /// Apply saturation adjustment to the entire image in place.
    /// saturation: 1.0 = no change, 0.0 = grayscale, 2.0 = doubled saturation
    pub fn apply_saturation(&mut self, saturation: f32) {
        if (saturation - 1.0).abs() < 0.01 {
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_pixel(x, y).unwrap();
                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;

                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                let l = (max + min) / 2.0;

                let s = if max == min {
                    0.0
                } else if l < 0.5 {
                    (max - min) / (max + min)
                } else {
                    (max - min) / (2.0 - max - min)
                };

                let h = if max == min {
                    0.0
                } else if max == r {
                    ((g - b) / (max - min) + if g < b { 6.0 } else { 0.0 }) / 6.0
                } else if max == g {
                    ((b - r) / (max - min) + 2.0) / 6.0
                } else {
                    ((r - g) / (max - min) + 4.0) / 6.0
                };

                let new_s = (s * saturation).min(1.0);

                let c = (1.0 - (2.0 * l - 1.0).abs()) * new_s;
                let x_val = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
                let m = l - c / 2.0;

                let (r_new, g_new, b_new) = match (h * 6.0) as i32 {
                    0 => (c, x_val, 0.0),
                    1 => (x_val, c, 0.0),
                    2 => (0.0, c, x_val),
                    3 => (0.0, x_val, c),
                    4 => (x_val, 0.0, c),
                    _ => (c, 0.0, x_val),
                };

                self.set_pixel(
                    x,
                    y,
                    [
                        ((r_new + m) * 255.0) as u8,
                        ((g_new + m) * 255.0) as u8,
                        ((b_new + m) * 255.0) as u8,
                        pixel[3],
                    ],
                )
                .ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation() {
        let img = Image::new_filled(256, 256, [255, 0, 0, 255]);
        assert_eq!(img.width, 256);
        assert_eq!(img.height, 256);
        assert_eq!(img.data.len(), 256 * 256 * 4);
    }

    #[test]
    fn test_image_fill_color() {
        let img = Image::new_filled(10, 10, [255, 128, 64, 200]);
        let pixel = img.get_pixel(5, 5).unwrap();
        assert_eq!(pixel, [255, 128, 64, 200]);
    }

    #[test]
    fn test_image_set_pixel() {
        let mut img = Image::new_filled(10, 10, [0, 0, 0, 0]);
        img.set_pixel(3, 4, [255, 128, 64, 200]).unwrap();
        assert_eq!(img.get_pixel(3, 4).unwrap(), [255, 128, 64, 200]);
    }

    #[test]
    fn test_image_set_pixel_bounds() {
        let mut img = Image::new_filled(10, 10, [0, 0, 0, 0]);
        assert!(img.set_pixel(10, 5, [255, 0, 0, 255]).is_err());
        assert!(img.set_pixel(5, 10, [255, 0, 0, 255]).is_err());
    }

    #[test]
    fn test_image_paste() {
        let mut dst = Image::new_filled(256, 256, [0, 0, 0, 255]);
        let src = Image::new_filled(16, 16, [255, 0, 0, 255]);

        dst.paste(0, 0, &src).unwrap();

        assert_eq!(dst.get_pixel(0, 0).unwrap(), [255, 0, 0, 255]);
        assert_eq!(dst.get_pixel(15, 15).unwrap(), [255, 0, 0, 255]);
        assert_eq!(dst.get_pixel(16, 16).unwrap(), [0, 0, 0, 255]);
    }

    #[test]
    fn test_image_paste_out_of_bounds() {
        let mut dst = Image::new_filled(10, 10, [0, 0, 0, 255]);
        let src = Image::new_filled(5, 5, [255, 0, 0, 255]);
        assert!(dst.paste(8, 8, &src).is_err());
    }

    #[test]
    fn test_image_paste_at_edge() {
        let mut dst = Image::new_filled(10, 10, [0, 0, 0, 255]);
        let src = Image::new_filled(5, 5, [255, 0, 0, 255]);
        assert!(dst.paste(5, 5, &src).is_ok());
    }

    #[test]
    fn test_from_raw() {
        let data = vec![128u8; 8 * 8 * 4];
        let img = Image::from_raw(8, 8, data).unwrap();
        assert_eq!(img.width, 8);
        assert_eq!(img.get_pixel(0, 0).unwrap(), [128, 128, 128, 128]);
    }

    #[test]
    fn test_from_raw_wrong_size() {
        let data = vec![0u8; 100]; // Wrong size for 8x8
        assert!(Image::from_raw(8, 8, data).is_err());
    }

    #[test]
    fn test_reduce_half_solid_color() {
        let img = Image::new_filled(8, 8, [200, 100, 50, 255]);
        let half = img.reduce_half();
        assert_eq!(half.width, 4);
        assert_eq!(half.height, 4);
        // Solid color should be preserved exactly
        assert_eq!(half.get_pixel(0, 0).unwrap(), [200, 100, 50, 255]);
        assert_eq!(half.get_pixel(3, 3).unwrap(), [200, 100, 50, 255]);
    }

    #[test]
    fn test_reduce_half_averaging() {
        // Create 2x2 image with known pixels
        let mut img = Image::new(2, 2);
        img.set_pixel(0, 0, [0, 0, 0, 255]).unwrap();
        img.set_pixel(1, 0, [100, 0, 0, 255]).unwrap();
        img.set_pixel(0, 1, [0, 100, 0, 255]).unwrap();
        img.set_pixel(1, 1, [0, 0, 100, 255]).unwrap();

        let half = img.reduce_half();
        assert_eq!(half.width, 1);
        assert_eq!(half.height, 1);
        // Average: R=(0+100+0+0)/4=25, G=(0+0+100+0)/4=25, B=(0+0+0+100)/4=25, A=255
        assert_eq!(half.get_pixel(0, 0).unwrap(), [25, 25, 25, 255]);
    }

    #[test]
    fn test_reduce_half_dimensions() {
        let img = Image::new_filled(256, 256, [0, 0, 0, 255]);
        let h1 = img.reduce_half();
        assert_eq!(h1.width, 128);
        assert_eq!(h1.height, 128);

        let h2 = h1.reduce_half();
        assert_eq!(h2.width, 64);
        assert_eq!(h2.height, 64);
    }

    #[test]
    fn test_reduce_n() {
        let img = Image::new_filled(256, 256, [100, 100, 100, 255]);
        let reduced = img.reduce_n(4); // 256 -> 128 -> 64 -> 32 -> 16
        assert_eq!(reduced.width, 16);
        assert_eq!(reduced.height, 16);
        assert_eq!(reduced.get_pixel(0, 0).unwrap(), [100, 100, 100, 255]);
    }

    #[test]
    fn test_reduce_to_minimum() {
        let img = Image::new_filled(4, 4, [80, 80, 80, 255]);
        let h1 = img.reduce_half(); // 2x2
        assert_eq!(h1.width, 2);
        let h2 = h1.reduce_half(); // 1x1
        assert_eq!(h2.width, 1);
        assert_eq!(h2.height, 1);
        assert_eq!(h2.get_pixel(0, 0).unwrap(), [80, 80, 80, 255]);
    }

    #[test]
    fn test_stride() {
        let img = Image::new(64, 32);
        assert_eq!(img.stride(), 64 * 4);
    }

    #[test]
    fn test_crop_basic() {
        let mut img = Image::new_filled(16, 16, [0, 0, 0, 255]);
        // Paint a 4x4 red square at (4, 4)
        for y in 4..8 {
            for x in 4..8 {
                img.set_pixel(x, y, [255, 0, 0, 255]).unwrap();
            }
        }

        let cropped = img.crop(4, 4, 4, 4).unwrap();
        assert_eq!(cropped.width, 4);
        assert_eq!(cropped.height, 4);
        assert_eq!(cropped.get_pixel(0, 0).unwrap(), [255, 0, 0, 255]);
        assert_eq!(cropped.get_pixel(3, 3).unwrap(), [255, 0, 0, 255]);
    }

    #[test]
    fn test_crop_out_of_bounds() {
        let img = Image::new(10, 10);
        assert!(img.crop(8, 8, 5, 5).is_err());
    }

    #[test]
    fn test_upscale_2x() {
        let img = Image::new_filled(2, 2, [100, 200, 50, 255]);
        let up = img.upscale(2);
        assert_eq!(up.width, 4);
        assert_eq!(up.height, 4);
        // All pixels should have the same color (nearest neighbor)
        assert_eq!(up.get_pixel(0, 0).unwrap(), [100, 200, 50, 255]);
        assert_eq!(up.get_pixel(3, 3).unwrap(), [100, 200, 50, 255]);
    }

    #[test]
    fn test_upscale_4x() {
        let mut img = Image::new(2, 2);
        img.set_pixel(0, 0, [255, 0, 0, 255]).unwrap();
        img.set_pixel(1, 0, [0, 255, 0, 255]).unwrap();
        img.set_pixel(0, 1, [0, 0, 255, 255]).unwrap();
        img.set_pixel(1, 1, [255, 255, 0, 255]).unwrap();

        let up = img.upscale(4);
        assert_eq!(up.width, 8);
        assert_eq!(up.height, 8);
        // Top-left 4x4 block should all be red
        assert_eq!(up.get_pixel(0, 0).unwrap(), [255, 0, 0, 255]);
        assert_eq!(up.get_pixel(3, 3).unwrap(), [255, 0, 0, 255]);
        // Top-right 4x4 block should all be green
        assert_eq!(up.get_pixel(4, 0).unwrap(), [0, 255, 0, 255]);
    }

    #[test]
    fn test_crop_and_upscale() {
        let img = Image::new_filled(256, 256, [128, 64, 32, 255]);
        // Crop 64x64 from (0,0), upscale 4x → 256x256
        let result = img.crop_and_upscale(0, 0, 64, 64, 4).unwrap();
        assert_eq!(result.width, 256);
        assert_eq!(result.height, 256);
        assert_eq!(result.get_pixel(0, 0).unwrap(), [128, 64, 32, 255]);
    }
}
