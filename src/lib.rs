use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

mod core;
use core::{
    PageData, 
    ImageOutputFormat
};


#[pyclass]
pub struct PyPageData {
    #[pyo3(get)]
    pub base64_image: String
}

// Implement conversion from PageData to PyPageData
impl From<PageData> for PyPageData {
    fn from(page: PageData) -> Self {
        Self {
            base64_image: page.base64_image
        }
    }
}

/// Converts a base64-encoded PDF string into a Python list of base64-encoded images (one per page)
/// 
/// Args:
///     base64_pdf (str): A base64-encoded string containing the PDF data
///     format (str): The format of the output images. Must be WEBP, PNG, or JPEG
///     quality (int): The quality of the output images. Must be between 0 and 100
///     max_edge_size (int): The maximum edge size of the output images. Must be between 1 and 10000
///     extract_text (bool): Whether to extract text from the PDF (not using OCR)
/// 
/// Returns:
///     List[PageData]: A list of PageData objects, each containing a base64-encoded image and optional text
/// 
/// Raises:
///     ValueError: If the PDF conversion fails
#[pyfunction]
pub fn render_base64_pdf(
    base64_pdf: String,
    format: &str,
    quality: u8,
    max_edge_size: i32
) -> PyResult<Vec<PyPageData>> {
    let format = format.parse::<ImageOutputFormat>()
        .map_err(|e| PyValueError::new_err(e))?;
    
    let result = core::render_base64_pdf(&base64_pdf, format, quality, max_edge_size)
        .map_err(|e| PyValueError::new_err(e))?;

    Ok(result.into_iter().map(Into::into).collect())
}

#[pyfunction]
pub fn compress_pdf(
    base64_pdf: String,
    quality: u8
) -> PyResult<String> {
    let compressed_pdf_base64 = core::compress_pdf(&base64_pdf, quality)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    
    Ok(compressed_pdf_base64)
}


/// A Python module implemented in Rust.
#[pymodule]
fn rust_ocr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render_base64_pdf, m)?)?;
    m.add_function(wrap_pyfunction!(compress_pdf, m)?)?;
    Ok(())
}