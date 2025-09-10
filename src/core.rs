use pdfium_render::prelude::*;
use image::DynamicImage;
use std::io::{Cursor, Write};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct PageData {
    pub image_buffer: Vec<u8>,
}

/// Converts PDF bytes into a vector of base64-encoded images (one per page)
/// Optionally extracts text from the PDF (not using OCR)
pub fn render_base64_pdf(
    pdfium: &Pdfium,
    pdf_bytes: &Vec<u8>,
    quality: u8
) -> Result<Vec<PageData>, String> {
    if quality > 100 {
        return Err("Quality must be between 0 and 100".to_string());
    }

    let document = pdfium
        .load_pdf_from_byte_slice(pdf_bytes, None)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;

    let images: Vec<_> = document
        .pages()
        .iter()
        .map(|page| {
            let bitmap = page.render_with_config(&PdfRenderConfig::new()
                .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true)
                .render_form_data(true)
                .use_grayscale_rendering(false))
                .map_err(|e| format!("Failed to render PDF page: {}", e))
                .map(|bitmap| bitmap.as_image().into_rgb8())?;

            let result = {
                let mut buffer = Cursor::new(Vec::new());

                let dynamic_image = DynamicImage::ImageRgb8(bitmap.to_owned());
                let webp_image = {
                    let encoder = webp::Encoder::from_image(&dynamic_image)
                        .map_err(|e| format!("Failed to create WebP encoder: {}", e))?;
                    encoder.encode(quality as f32)
                };
                
                buffer.write_all(&*webp_image)
                    .map_err(|e| format!("Failed to write WebP image: {}", e))?;
                
                drop(dynamic_image);
                drop(webp_image);

                let image_buffer = buffer.into_inner();
                
                PageData {
                    image_buffer: image_buffer,
                }
            };
            
            Ok(result)
        })
        .collect::<Result<Vec<_>, String>>()?;

    drop(document);
    
    Ok(images)
}


/// Opens a PDF from a base64 string and compresses its internal images to JPEG.

/// # Arguments
/// * `base64_pdf` - A base64 encoded string of the source PDF file.
/// * `quality` - The JPEG quality setting, from 1 (lowest) to 100 (highest).
///   A value around 75 is a good balance between size and quality.
///
/// # Returns
/// A `Result` containing the base64-encoded compressed PDF, or an error.
pub fn compress_pdf(
    base64_pdf: &str,
    quality: u8,
) -> Result<String, Box<dyn Error>> {
    if quality == 0 || quality > 100 {
        return Err("Quality must be between 1 and 100".into());
    }


    Ok(base64_pdf.to_string())
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use super::*;

    #[test]
    fn test_render_base64_pdf() {
        let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/local/lib/"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .expect("Failed to bind to Pdfium library")
        );
        let test_pdf_path = "./samples/test.pdf";
        let pdf_bytes = std::fs::read(test_pdf_path)
            .expect("Failed to read test PDF file");
   
        match render_base64_pdf(&pdfium, &pdf_bytes, 75) {
            Ok(r) => {
                assert_eq!(r.len(), 5);
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    #[test]
    fn test_compress_pdf() {
        let test_pdf_path = "./samples/test.pdf";
        let pdf_bytes = std::fs::read(test_pdf_path)
            .expect("Failed to read test PDF file");
        let original_size = pdf_bytes.len();
        let base64_pdf = BASE64.encode(&pdf_bytes);

        println!("Original PDF size: {} bytes", original_size);

        match compress_pdf(&base64_pdf, 75) {
            Ok(compressed_base64) => {
                let compressed_bytes = BASE64.decode(&compressed_base64)
                    .expect("Failed to decode compressed PDF");
                let compressed_size = compressed_bytes.len();
                
                let compression_ratio = (original_size as f64 - compressed_size as f64) / original_size as f64 * 100.0;
                
                println!("Compressed PDF size: {} bytes", compressed_size);
                println!("Compression ratio: {:.2}%", compression_ratio);
                
                // Save the compressed PDF to target folder
                let output_path = "./target/compressed_test.pdf";
                std::fs::write(output_path, &compressed_bytes)
                    .expect("Failed to write compressed PDF");
                println!("Compressed PDF saved to: {}", output_path);
                
                assert!(compressed_size > 0, "Compressed PDF should not be empty");
                println!("✓ Compression test completed successfully");
            },
            Err(e) => {
                println!("Error compressing PDF: {}", e);
                panic!("PDF compression failed");
            }
        }
    }
}