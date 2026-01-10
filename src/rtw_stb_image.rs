use std::path::Path;

pub struct RtwImage {
    bdata: Option<Vec<u8>>,
    image_width: i32,
    image_height: i32,
    bytes_per_scanline: i32,
    bytes_per_pixel: i32,
}

impl RtwImage {
    pub fn new() -> Self {
        Self {
            bdata: None,
            image_width: 0,
            image_height: 0,
            bytes_per_scanline: 0,
            bytes_per_pixel: 3,
        }
    }

    pub fn load(filename: &str) -> Self {
        let mut image = Self::new();
        
        // Hunt for the image file in some likely locations
        let search_paths = [
            filename.to_string(),
            format!("images/{}", filename),
            format!("../images/{}", filename),
            format!("../../images/{}", filename),
            format!("../../../images/{}", filename),
            format!("../../../../images/{}", filename),
            format!("../../../../../images/{}", filename),
            format!("../../../../../../images/{}", filename),
        ];

        // Check RTW_IMAGES environment variable first
        if let Ok(imagedir) = std::env::var("RTW_IMAGES") {
            let env_path = format!("{}/{}", imagedir, filename);
            if image.load_from_file(&env_path) {
                return image;
            }
        }

        // Try each search path
        for path in &search_paths {
            if image.load_from_file(path) {
                return image;
            }
        }

        eprintln!("ERROR: Could not load image file '{}'.", filename);
        image
    }

    fn load_from_file(&mut self, filename: &str) -> bool {
        eprintln!("Trying to load: {}", filename);
        
        // Load image using the image crate
        let img = match image::open(Path::new(filename)) {
            Ok(img) => {
                eprintln!("Successfully loaded: {}", filename);
                img
            },
            Err(e) => {
                eprintln!("Failed to load {}: {}", filename, e);
                return false;
            }
        };

        let rgb_img = img.to_rgb8();
        self.image_width = rgb_img.width() as i32;
        self.image_height = rgb_img.height() as i32;
        self.bytes_per_scanline = self.image_width * self.bytes_per_pixel;
        
        eprintln!("Image dimensions: {}x{}", self.image_width, self.image_height);
        
        // Convert to our internal format
        self.bdata = Some(rgb_img.into_raw());
        
        true
    }

    pub fn width(&self) -> i32 {
        if self.bdata.is_none() {
            0
        } else {
            self.image_width
        }
    }

    pub fn height(&self) -> i32 {
        if self.bdata.is_none() {
            0
        } else {
            self.image_height
        }
    }

    pub fn pixel_data(&self, x: i32, y: i32) -> &[u8] {
        // Return the address of the three RGB bytes of the pixel at x,y
        // If there is no image data, returns magenta
        static MAGENTA: [u8; 3] = [255, 0, 255];
        
        match &self.bdata {
            None => &MAGENTA,
            Some(data) => {
                let x = Self::clamp(x, 0, self.image_width);
                let y = Self::clamp(y, 0, self.image_height);
                
                let offset = (y * self.bytes_per_scanline + x * self.bytes_per_pixel) as usize;
                &data[offset..offset + self.bytes_per_pixel as usize]
            }
        }
    }

    fn clamp(x: i32, low: i32, high: i32) -> i32 {
        if x < low {
            low
        } else if x < high {
            x
        } else {
            high - 1
        }
    }
}