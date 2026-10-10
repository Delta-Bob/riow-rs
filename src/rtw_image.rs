//! Minimal image loading for `ImageTexture`.

use std::path::Path;

const BYTES_PER_PIXEL: i32 = 3;

/// An RGB image, or an empty placeholder if loading failed.
pub struct RtwImage {
    data: Option<Vec<u8>>,
    width: i32,
    height: i32,
}

impl RtwImage {
    /// Loads an image, searching the `RTW_IMAGES` directory (if set), then the
    /// current directory, then the usual relative `images/` locations.
    pub fn load(filename: &str) -> Self {
        let mut candidates = Vec::new();
        if let Ok(dir) = std::env::var("RTW_IMAGES") {
            candidates.push(format!("{dir}/{filename}"));
        }
        candidates.push(filename.to_string());
        for prefix in [
            "images",
            "../images",
            "../../images",
            "../../../images",
            "../../../../images",
            "../../../../../images",
            "../../../../../../images",
        ] {
            candidates.push(format!("{prefix}/{filename}"));
        }

        for path in candidates {
            if let Some(image) = Self::load_from_file(&path) {
                return image;
            }
        }

        eprintln!("ERROR: Could not load image file '{filename}'.");
        Self::empty()
    }

    fn empty() -> Self {
        Self {
            data: None,
            width: 0,
            height: 0,
        }
    }

    fn load_from_file(filename: &str) -> Option<Self> {
        let img = image::open(Path::new(filename)).ok()?;
        let rgb = img.to_rgb8();
        let width = rgb.width() as i32;
        let height = rgb.height() as i32;

        eprintln!("Loaded image '{filename}' ({width}x{height})");

        Some(Self {
            data: Some(rgb.into_raw()),
            width,
            height,
        })
    }

    /// Image width in pixels, or 0 if no image was loaded.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Image height in pixels, or 0 if no image was loaded.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// The three RGB bytes of the pixel at `(x, y)`, clamped to the image
    /// bounds. Returns magenta when no image was loaded.
    pub fn pixel_data(&self, x: i32, y: i32) -> &[u8] {
        static MAGENTA: [u8; 3] = [255, 0, 255];

        let Some(data) = &self.data else {
            return &MAGENTA;
        };

        let x = x.clamp(0, self.width - 1);
        let y = y.clamp(0, self.height - 1);
        let offset = (y * self.width * BYTES_PER_PIXEL + x * BYTES_PER_PIXEL) as usize;

        &data[offset..offset + BYTES_PER_PIXEL as usize]
    }
}
