#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "uptime-kuma-api>=1.2.0",
# ]
# ///
"""Export Uptime Kuma monitors to JSON for import into admin app.

Usage:
    uv run scripts/kuma_export.py --url https://kuma.example.com --username admin --password secret

Output:
    Creates kuma-monitors.json with HTTP/keyword monitors formatted for the admin app import wizard.
"""

import json
import argparse
import sys
from urllib.parse import urlparse

from uptime_kuma_api import UptimeKumaApi


def export_monitors(kuma_url: str, username: str, password: str, output: str) -> None:
    """Connect to Uptime Kuma and export HTTP monitors to JSON.

    Args:
        kuma_url: Base URL of the Uptime Kuma instance
        username: Login username
        password: Login password
        output: Output file path for the JSON export
    """
    print(f"Connecting to {kuma_url}...")

    api = UptimeKumaApi(kuma_url)

    try:
        api.login(username, password)
        print("Login successful.")

        monitors = api.get_monitors()
        print(f"Found {len(monitors)} total monitors.")

        # Filter to HTTP/HTTPS monitors and transform to import format
        export_data = []
        skipped = 0

        for m in monitors:
            monitor_type = m.get("type")

            # Only process HTTP-based monitor types
            # Types: http, keyword (HTTP with keyword check)
            if monitor_type not in ["http", "keyword"]:
                skipped += 1
                continue

            url = m.get("url", "")
            if not url:
                print(f"  Skipping monitor '{m.get('name')}': no URL")
                skipped += 1
                continue

            parsed = urlparse(url)

            # Parse expected status - can be a single number or comma-separated
            expected_status_raw = m.get("accepted_statuscodes") or ["200"]
            if isinstance(expected_status_raw, list) and expected_status_raw:
                # Take the first expected status code
                try:
                    expected_status = int(expected_status_raw[0].split("-")[0])
                except (ValueError, IndexError):
                    expected_status = 200
            else:
                expected_status = 200

            # Parse timeout (Kuma stores in milliseconds, we want seconds)
            timeout_ms = m.get("timeout") or 30000
            timeout_seconds = max(1, min(300, timeout_ms // 1000))

            # Headers are stored as JSON string in Kuma
            headers = m.get("headers")
            if headers and isinstance(headers, str):
                try:
                    # Validate it's proper JSON
                    json.loads(headers)
                except json.JSONDecodeError:
                    headers = None

            # Parse retry settings from Kuma (maxretries field)
            max_retries = m.get("maxretries") or 0
            retry_interval = m.get("retryInterval") or 60

            # Parse HTTP body for POST requests
            request_body = m.get("body")
            # Kuma uses "json" for JSON bodies
            body_encoding = "JSON"
            if m.get("httpBodyEncoding"):
                encoding = m.get("httpBodyEncoding").lower()
                if encoding == "form":
                    body_encoding = "x-www-form-urlencoded"
                elif encoding == "xml":
                    body_encoding = "XML"

            # Parse HTTP Basic Auth
            http_auth_user = m.get("basic_auth_user")
            http_auth_pass = m.get("basic_auth_pass")

            export_data.append(
                {
                    "kuma_id": m.get("id"),
                    "name": m.get("name") or f"Monitor {m.get('id')}",
                    "url": url,
                    "hostname": parsed.netloc or parsed.hostname or "",
                    "path": parsed.path or "/",
                    "protocol": parsed.scheme or "https",
                    "method": (m.get("method") or "GET").upper(),
                    "expected_status": expected_status,
                    "timeout_seconds": timeout_seconds,
                    "headers": headers,
                    "keyword": m.get("keyword"),  # For body match (keyword monitors)
                    "retry": max_retries,
                    "retry_interval": retry_interval,
                    "request_body": request_body,
                    "request_body_encoding": body_encoding,
                    "http_auth_user": http_auth_user,
                    "http_auth_pass": http_auth_pass,
                }
            )

        print(f"Skipped {skipped} non-HTTP monitors.")

    finally:
        api.disconnect()
        print("Disconnected from Uptime Kuma.")

    # Write output
    with open(output, "w") as f:
        json.dump(export_data, f, indent=2)

    print(f"\nExported {len(export_data)} monitors to {output}")

    # Show preview
    if export_data:
        print("\nPreview of exported monitors:")
        for m in export_data[:5]:
            print(f"  - {m['name']}: {m['url']}")
        if len(export_data) > 5:
            print(f"  ... and {len(export_data) - 5} more")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Export Uptime Kuma monitors to JSON for admin app import",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    %(prog)s --url https://kuma.example.com --username admin --password secret
    %(prog)s --url http://localhost:3001 -u admin -p pass -o monitors.json
        """,
    )
    parser.add_argument(
        "--url", "-U", required=True, help="Uptime Kuma URL (e.g., https://kuma.example.com)"
    )
    parser.add_argument("--username", "-u", required=True, help="Kuma username")
    parser.add_argument("--password", "-p", required=True, help="Kuma password")
    parser.add_argument(
        "--output", "-o", default="kuma-monitors.json", help="Output file (default: kuma-monitors.json)"
    )

    args = parser.parse_args()

    try:
        export_monitors(args.url, args.username, args.password, args.output)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
