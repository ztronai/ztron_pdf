mod core;
use std::thread;
use pdfium_render::prelude::*;

fn main() {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("/usr/local/lib/"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .expect("Failed to bind to Pdfium library")
    );

    let test_pdf_path = "./samples/test2.pdf";
    let pdf_bytes = std::fs::read(test_pdf_path)
        .expect("Failed to read test PDF file");

    for _ in 0..3 {
        match core::render_base64_pdf(&pdfium, &pdf_bytes, 75) {
            Ok(r) => {
                assert!(r.len() > 0);
                println!("Rendered {} images", r.len());
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }

    pdfium.bindings().FPDF_DestroyLibrary();
    drop(pdfium);

    thread::sleep(std::time::Duration::from_secs(2));
    std::process::exit(0);
}