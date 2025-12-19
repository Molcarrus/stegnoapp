use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use image::{ImageBuffer, Rgb};

use crate::errors::Error;
use crate::utils::ByteMask;

pub struct Decoder {
    image: ImageBuffer<Rgb<u8>, Vec<u8>>,
    mask: ByteMask
}

impl Decoder {
    pub fn new(
        image_path: PathBuf,
        mask: ByteMask
    ) -> Result<Self, Error> {
        let image = image::open(image_path)?.to_rgb8();
        
        Ok(Decoder { image, mask })
    }
    
    pub fn save(&self, output: PathBuf) -> Result<(), Error> {
        let mut secret = BufWriter::new(File::create(output)?);
        let mut chunks = Vec::with_capacity(self.mask.chunks as usize);
        let mut start = false;
        
        for (i, b) in self.image.iter().map(|b| b & self.mask.mask).enumerate() {
            if !start && (b > 0) {
                let n = self.mask.chunks as usize;
                let offset = (self.image.len() - i) % n;
                if offset != 0 {
                    (0..(n - offset)).for_each(|_| chunks.push(0));
                }
                start = true;
            };
            
            if start {
                chunks.push(b);
            }
            
            if chunks.len() == chunks.capacity() {
                let byte = self.mask.join_chunks(&chunks);
                secret.write_all(&[byte])?;
                chunks.clear();
            }
        }
        
        secret.flush()?;
        Ok(())
    }
}