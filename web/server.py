# TODO: convert to use pxs

from http.server import HTTPServer, SimpleHTTPRequestHandler


class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()


if __name__ == "__main__":
    server_address = ("", 8000)
    httpd = HTTPServer(server_address, CORSRequestHandler)
    print("Serving with COOP/COEP headers on http://localhost:8000")
    httpd.serve_forever()
