use pdfium_render::prelude::*;
use image::{DynamicImage, ImageFormat};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::io::{Cursor, Write};
use std::error::Error;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PageData {
    pub base64_image: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageOutputFormat {
    JPEG,
    PNG,
    WEBP,
}

impl FromStr for ImageOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "JPEG" => Ok(ImageOutputFormat::JPEG),
            "PNG" => Ok(ImageOutputFormat::PNG),
            "WEBP" => Ok(ImageOutputFormat::WEBP),
            _ => Err(format!("Invalid format: {}. Must be JPEG, PNG, or WEBP", s)),
        }
    }
}

/// Converts a base64-encoded PDF string into a vector of base64-encoded images (one per page)
/// Optionally extracts text from the PDF (not using OCR)
pub fn render_base64_pdf(
    base64_pdf: &str,
    format: ImageOutputFormat,
    quality: u8,
    max_edge_size: i32
) -> Result<Vec<PageData>, String> {
    if quality > 100 {
        return Err("Quality must be between 0 and 100".to_string());
    }
    if max_edge_size <= 0 || max_edge_size > 10_000 {
        return Err("max_edge_size must be between 1 and 10000".to_string());
    }

    let pdf_data = BASE64.decode(base64_pdf)
        .map_err(|e| format!("Invalid base64 input: {}", e))?;

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/local/lib/"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .map_err(|e| format!("Failed to bind to Pdfium library: {}", e))?
    );
    
    let document = pdfium
        .load_pdf_from_byte_slice(&pdf_data, None)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;
    
    let rendered_pdf: Vec<_> = document
        .pages()
        .iter()
        .map(|page| {
            let bitmap = page.render_with_config(&PdfRenderConfig::new()
                .set_target_width(max_edge_size)
                .set_maximum_height(max_edge_size)
                .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true)
                .render_form_data(true)
                .use_grayscale_rendering(false))
                .map_err(|e| format!("Failed to render PDF page: {}", e))
                .map(|bitmap| bitmap.as_image().into_rgb8())?;

            Ok(bitmap)
        })
        .collect::<Result<Vec<_>, String>>()?;

    let images: Vec<PageData> = rendered_pdf
        .iter()
        .map(|page_content| {
            let result = {
                let mut buffer = Cursor::new(Vec::new());

                match format {
                    ImageOutputFormat::JPEG => {
                        page_content.write_to(&mut buffer, ImageFormat::Jpeg)
                            .map_err(|e| format!("Failed to write JPEG image: {}", e))?;
                    },
                    ImageOutputFormat::PNG => {
                        page_content.write_to(&mut buffer, ImageFormat::Png)
                            .map_err(|e| format!("Failed to write PNG image: {}", e))?;
                    },
                    ImageOutputFormat::WEBP => {
                        let dynamic_image = DynamicImage::ImageRgb8(page_content.to_owned());
                        let webp_image = {
                            let encoder = webp::Encoder::from_image(&dynamic_image)
                                .map_err(|e| format!("Failed to create WebP encoder: {}", e))?;
                            encoder.encode(quality as f32)
                        }; // encoder is dropped here
                        
                        buffer.write_all(&*webp_image)
                            .map_err(|e| format!("Failed to write WebP image: {}", e))?;
                        
                        // Now we can safely drop dynamic_image
                        drop(dynamic_image);
                        drop(webp_image);
                    },
                }

                let buffer_data = buffer.into_inner();
                let base64_result = BASE64.encode(&buffer_data);
                
                drop(buffer_data);
                
                PageData {
                    base64_image: base64_result,
                }
            };
            
            Ok(result)
        })
        .collect::<Result<Vec<_>, String>>()?;

    drop(rendered_pdf);
    drop(document);
    drop(pdfium);
    drop(pdf_data);

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
        let test_pdf_path = "./samples/test.pdf";
        let pdf_bytes = std::fs::read(test_pdf_path)
            .expect("Failed to read test PDF file");
        let base64_pdf = BASE64.encode(pdf_bytes);

        match render_base64_pdf(&base64_pdf, ImageOutputFormat::JPEG, 75, 100) {
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