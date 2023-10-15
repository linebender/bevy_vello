use bevy::log;
use std::io::Read;

/// Deflate gzip compression to plaintext
pub fn decompress_gzip(content: &[u8]) -> String {
    #[cfg(debug_assertions)]
    {
        log::info!("original (gzipped) size: {} bytes", content.len());
    }
    let mut d = flate2::read::GzDecoder::new(content);
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    #[cfg(debug_assertions)]
    {
        let new_bytes = s.as_bytes();
        let after_percent = new_bytes.len() as f32 / content.len() as f32;
        log::info!(
            "decompressed size: {} bytes ({:.2}%)",
            new_bytes.len(),
            after_percent
        );
    }
    s
}
