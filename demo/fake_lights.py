"""Fake Elgato lights for the demo. Serves the /elgato/lights API on localhost."""

import json
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

LIGHTS = [
    ("Elgato Key Light Air 7C4F", 19123),
    ("Elgato Key Light 2B91", 19124),
]


def make_handler(state):
    class Handler(BaseHTTPRequestHandler):
        def _send(self):
            body = json.dumps({"numberOfLights": 1, "lights": [state]}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            self._send()

        def do_PUT(self):
            length = int(self.headers.get("Content-Length", 0))
            update = json.loads(self.rfile.read(length))["lights"][0]
            state.update(update)
            self._send()

        def log_message(self, *args):
            pass

    return Handler


def main():
    cache_file = sys.argv[1]
    servers = []
    cache = []
    for name, port in LIGHTS:
        state = {"on": 0, "brightness": 20, "temperature": 200}
        servers.append(ThreadingHTTPServer(("127.0.0.1", port), make_handler(state)))
        cache.append({"name": name, "ip": "127.0.0.1", "port": port})

    # The CLI reads discovered lights from this cache, so it skips Bonjour.
    with open(cache_file, "w") as f:
        json.dump(cache, f)

    for server in servers[1:]:
        threading.Thread(target=server.serve_forever, daemon=True).start()
    servers[0].serve_forever()


if __name__ == "__main__":
    main()
