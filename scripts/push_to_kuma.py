#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "requests>=2.23.5",
#     "uptime-kuma-api>=1.2.0",
# ]
# ///
import argparse
import sys

import requests
from uptime_kuma_api import UptimeKumaApi


def push_to_kuma(username: str, password: str):
    # Fetch monitors from admin API
    monitors = requests.get(
        "http://auto.ghentcdh.be/api/healthchecks/export/kuma"
    ).json()

    # Push to Kuma
    api = UptimeKumaApi("https://dash.ghentcdh.be")
    api.login(username, password)

    for m in monitors:
        print(f"{m}")
        # api.add_monitor(
        #     type=MonitorType.HTTP,
        #     name=m["name"],
        #     url=m["url"],
        #     method=m["method"],
        #     expected_status=m["expected_status"],
        #     timeout=m["timeout"],
        # )

    api.disconnect()


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Get Auto healthchecks and push then to Kuma",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--username", "-u", required=True, help="Kuma username")
    parser.add_argument("--password", "-p", required=True, help="Kuma password")

    args = parser.parse_args()

    try:
        push_to_kuma(args.username, args.password)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
