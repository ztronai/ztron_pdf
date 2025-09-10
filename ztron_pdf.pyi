from typing import List

class PyPageData:
    image_buffer: bytes

def render_base64_pdf(pdf_bytes: bytes, quality: int) -> List[PyPageData]: ...

def compress_pdf(base64_pdf: str, quality: int) -> str: ...