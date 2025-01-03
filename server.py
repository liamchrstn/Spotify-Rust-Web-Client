from http.server import HTTPServer, SimpleHTTPRequestHandler
import os

class WASMHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory="pkg", **kwargs)

    def end_headers(self):
        # Enable CORS
        self.send_header('Access-Control-Allow-Origin', '*')
        # Add WASM MIME type
        if self.path.endswith('.wasm'):
            self.send_header('Content-Type', 'application/wasm')
        super().end_headers()

if __name__ == '__main__':
    port = 8000
    print(f"Serving on port {port}")
    httpd = HTTPServer(('', port), WASMHandler)
    httpd.serve_forever()
