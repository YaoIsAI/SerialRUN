#!/usr/bin/env python3
"""
GitHub Webhook server for SerialRUN plugin development.
Receives issue events and creates plugin development requests.

Usage:
    python3 scripts/webhook_server.py --port 9876 --secret YOUR_SECRET

GitHub Webhook setup:
    1. Go to your repo → Settings → Webhooks → Add webhook
    2. Payload URL: http://YOUR_IP:9876/webhook
    3. Content type: application/json
    4. Secret: YOUR_SECRET
    5. Events: Issues
"""

import argparse
import hashlib
import hmac
import json
import os
import sys
from datetime import datetime
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

# Config
PENDING_DIR = Path.home() / ".serialrun" / "pending_plugins"
LOG_FILE = Path.home() / ".serialrun" / "webhook.log"

class WebhookHandler(BaseHTTPRequestHandler):
    def __init__(self, *args, secret=None, **kwargs):
        self.secret = secret
        super().__init__(*args, **kwargs)

    def do_POST(self):
        if self.path != "/webhook":
            self.send_error(404)
            return

        # Read body
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        # Verify signature
        if self.secret:
            signature = self.headers.get("X-Hub-Signature-256", "")
            expected = "sha256=" + hmac.new(
                self.secret.encode(), body, hashlib.sha256
            ).hexdigest()
            if not hmac.compare_digest(signature, expected):
                self.send_error(403, "Invalid signature")
                return

        # Parse event
        event = self.headers.get("X-GitHub-Event", "")
        if event != "issues":
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'{"status": "ignored"}')
            return

        payload = json.loads(body)
        action = payload.get("action", "")

        if action != "opened":
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'{"status": "ignored"}')
            return

        issue = payload.get("issue", {})
        self.process_plugin_request(issue)

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps({"status": "processed"}).encode())

    def process_plugin_request(self, issue):
        title = issue.get("title", "")
        body = issue.get("body", "")
        number = issue.get("number", 0)
        labels = [l["name"] for l in issue.get("labels", [])]

        # Check if it's a plugin request
        if "plugin-request" not in labels and "插件需求" not in labels:
            self.log(f"Issue #{number} doesn't have plugin-request label, skipping")
            return

        # Parse plugin info from issue
        plugin_info = self.parse_issue(title, body)

        # Create pending request
        PENDING_DIR.mkdir(parents=True, exist_ok=True)
        request_file = PENDING_DIR / f"issue-{number}.json"

        request = {
            "issue_number": number,
            "issue_url": issue.get("html_url", ""),
            "title": title,
            "created_at": datetime.now().isoformat(),
            "plugin_name": plugin_info["name"],
            "description": plugin_info["description"],
            "features": plugin_info["features"],
            "protocol": plugin_info.get("protocol", ""),
            "status": "pending",
        }

        request_file.write_text(json.dumps(request, indent=2, ensure_ascii=False))
        self.log(f"Created plugin request: {request_file.name}")
        self.log(f"  Plugin: {plugin_info['name']}")
        self.log(f"  Features: {plugin_info['features']}")

    def parse_issue(self, title, body):
        """Parse plugin info from issue title and body."""
        # Extract plugin name from title
        name = title.lower().replace(" ", "-").replace("[", "").replace("]", "")
        name = f"serialrun-{name}"

        # Parse body sections
        description = ""
        features = []
        protocol = ""

        current_section = ""
        for line in body.split("\n"):
            line = line.strip()
            if line.startswith("## "):
                current_section = line[3:].lower()
            elif current_section == "description" and line:
                description = line
            elif current_section == "features" and line.startswith("- "):
                features.append(line[2:])
            elif current_section == "protocol" and line:
                protocol = line

        return {
            "name": name,
            "description": description or title,
            "features": features or ["basic serial communication"],
            "protocol": protocol,
        }

    def log(self, msg):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        line = f"[{timestamp}] {msg}"
        print(line)
        with open(LOG_FILE, "a") as f:
            f.write(line + "\n")

    def log_message(self, format, *args):
        """Override to suppress default logging."""
        pass

def main():
    parser = argparse.ArgumentParser(description="GitHub Webhook server for SerialRUN")
    parser.add_argument("--port", type=int, default=9876, help="Port to listen on")
    parser.add_argument("--secret", type=str, help="Webhook secret")
    args = parser.parse_args()

    PENDING_DIR.mkdir(parents=True, exist_ok=True)

    print(f"Starting webhook server on port {args.port}")
    print(f"Pending plugins dir: {PENDING_DIR}")
    print(f"Log file: {LOG_FILE}")
    print()

    # Create handler with secret
    handler = lambda *a, **kw: WebhookHandler(*a, secret=args.secret, **kw)

    server = HTTPServer(("0.0.0.0", args.port), handler)
    print(f"Listening on 0.0.0.0:{args.port}")
    print("Press Ctrl+C to stop")
    print()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()

if __name__ == "__main__":
    main()
