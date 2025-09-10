from typing import List

class PyPageData:
    base64_image: str

def render_base64_pdf(base64_pdf: str, format: str, quality: int, max_edge_size: int) -> List[PyPageData]: ...

def compress_pdf(base64_pdf: str, quality: int) -> str: ...