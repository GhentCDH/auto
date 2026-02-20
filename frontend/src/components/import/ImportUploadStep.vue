<script setup lang="ts">
import { ref } from 'vue';
import type { KumaMonitorImport } from '@/types';
import { Copy, Download, Check, ChevronDown, ChevronUp } from 'lucide-vue-next';

const emit = defineEmits<{
  parsed: [monitors: KumaMonitorImport[]];
  cancel: [];
}>();

const fileInput = ref<HTMLInputElement | null>(null);
const fileName = ref('');
const error = ref('');
const previewCount = ref(0);

// Script section state
const scriptExpanded = ref(false);
const copied = ref(false);

const KUMA_EXPORT_SCRIPT = `#!/usr/bin/env -S uv run
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

    print(f"\\nExported {len(export_data)} monitors to {output}")

    # Show preview
    if export_data:
        print("\\nPreview of exported monitors:")
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
`;

async function copyScript() {
  try {
    await navigator.clipboard.writeText(KUMA_EXPORT_SCRIPT);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch {
    // Fallback for older browsers
    const textarea = document.createElement('textarea');
    textarea.value = KUMA_EXPORT_SCRIPT;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  }
}

function downloadScript() {
  const blob = new Blob([KUMA_EXPORT_SCRIPT], { type: 'text/x-python' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'kuma_export.py';
  a.click();
  URL.revokeObjectURL(url);
}

function validateMonitor(m: unknown, index: number): KumaMonitorImport {
  if (typeof m !== 'object' || m === null) {
    throw new Error(`Item ${index} is not an object`);
  }

  const monitor = m as Record<string, unknown>;

  // Required fields
  if (typeof monitor.name !== 'string' || !monitor.name) {
    throw new Error(`Item ${index}: missing or invalid 'name'`);
  }
  if (typeof monitor.url !== 'string' || !monitor.url) {
    throw new Error(`Item ${index}: missing or invalid 'url'`);
  }
  if (typeof monitor.hostname !== 'string' || !monitor.hostname) {
    throw new Error(`Item ${index}: missing or invalid 'hostname'`);
  }

  return {
    kuma_id: typeof monitor.kuma_id === 'number' ? monitor.kuma_id : index,
    name: monitor.name,
    url: monitor.url,
    hostname: monitor.hostname,
    path: typeof monitor.path === 'string' ? monitor.path : '/',
    protocol: typeof monitor.protocol === 'string' ? monitor.protocol : 'https',
    method: typeof monitor.method === 'string' ? monitor.method : 'GET',
    expected_status:
      typeof monitor.expected_status === 'number'
        ? monitor.expected_status
        : 200,
    timeout_seconds:
      typeof monitor.timeout_seconds === 'number'
        ? monitor.timeout_seconds
        : 30,
    interval:
      typeof monitor.interval === 'number' ? monitor.interval : 60,
    headers: typeof monitor.headers === 'string' ? monitor.headers : null,
    keyword: typeof monitor.keyword === 'string' ? monitor.keyword : null,
    retry: typeof monitor.retry === 'number' ? monitor.retry : 0,
    retry_interval:
      typeof monitor.retry_interval === 'number' ? monitor.retry_interval : 60,
    request_body:
      typeof monitor.request_body === 'string' ? monitor.request_body : null,
    request_body_encoding:
      typeof monitor.request_body_encoding === 'string'
        ? monitor.request_body_encoding
        : 'JSON',
    http_auth_user:
      typeof monitor.http_auth_user === 'string' ? monitor.http_auth_user : null,
    http_auth_pass:
      typeof monitor.http_auth_pass === 'string' ? monitor.http_auth_pass : null,
  };
}

async function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];

  if (!file) return;

  fileName.value = file.name;
  error.value = '';
  previewCount.value = 0;

  if (!file.name.endsWith('.json')) {
    error.value = 'Please select a JSON file';
    return;
  }

  try {
    const text = await file.text();
    const parsed = JSON.parse(text);

    if (!Array.isArray(parsed)) {
      throw new Error('Expected an array of monitors');
    }

    if (parsed.length === 0) {
      throw new Error('No monitors found in the file');
    }

    // Validate and transform each monitor
    const monitors: KumaMonitorImport[] = [];
    for (let i = 0; i < parsed.length; i++) {
      monitors.push(validateMonitor(parsed[i], i));
    }

    previewCount.value = monitors.length;

    // Store for later use when clicking "Next"
    (window as { __kumaMonitors?: KumaMonitorImport[] }).__kumaMonitors =
      monitors;
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to parse JSON file';
    previewCount.value = 0;
  }
}

function handleNext() {
  const monitors = (window as { __kumaMonitors?: KumaMonitorImport[] })
    .__kumaMonitors;
  if (monitors && monitors.length > 0) {
    emit('parsed', monitors);
    delete (window as { __kumaMonitors?: KumaMonitorImport[] }).__kumaMonitors;
  }
}

function triggerFileSelect() {
  fileInput.value?.click();
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <div class="text-center">
      <h3 class="text-lg font-medium mb-2">Upload Kuma Export</h3>
      <p class="text-base-content/70 text-sm">
        Select a JSON file exported from Uptime Kuma using the export script
      </p>
    </div>

    <!-- Export Script Section -->
    <div class="collapse bg-base-200 rounded-box">
      <input type="checkbox" v-model="scriptExpanded" />
      <div
        class="collapse-title text-sm py-2 min-h-0 flex items-center gap-x-2"
      >
        <span class="text-base-content/70">Need the export script? </span>
        <ChevronDown v-if="!scriptExpanded" class="w-4 h-4" />
        <ChevronUp v-if="scriptExpanded" class="w-4 h-4" />
      </div>
      <div class="collapse-content">
        <div class="flex gap-2 mb-3">
          <button
            type="button"
            class="btn btn-sm btn-ghost gap-1"
            @click.stop="copyScript"
          >
            <Check v-if="copied" class="w-4 h-4 text-success" />
            <Copy v-else class="w-4 h-4" />
            {{ copied ? 'Copied!' : 'Copy' }}
          </button>
          <button
            type="button"
            class="btn btn-sm btn-ghost gap-1"
            @click.stop="downloadScript"
          >
            <Download class="w-4 h-4" />
            Download
          </button>
        </div>
        <pre
          class="bg-base-300 rounded-box p-3 text-xs overflow-x-auto max-h-64 overflow-y-auto"
        ><code>{{ KUMA_EXPORT_SCRIPT }}</code></pre>
      </div>
    </div>

    <div
      class="border-2 border-dashed border-base-300 rounded-box p-8 text-center hover:border-primary transition-colors cursor-pointer"
      @click="triggerFileSelect"
    >
      <input
        ref="fileInput"
        type="file"
        accept=".json"
        class="hidden"
        @change="handleFileSelect"
      />

      <div v-if="!fileName" class="space-y-2">
        <div class="text-4xl">📄</div>
        <p class="text-base-content/70">
          Click to select <span class="font-mono">kuma-monitors.json</span>
        </p>
      </div>

      <div v-else class="space-y-2">
        <div class="text-4xl">✅</div>
        <p class="font-medium">{{ fileName }}</p>
        <p v-if="previewCount > 0" class="text-success text-sm">
          Found {{ previewCount }} monitor{{ previewCount !== 1 ? 's' : '' }} to
          import
        </p>
      </div>
    </div>

    <div v-if="error" class="alert alert-error text-sm">
      {{ error }}
    </div>

    <div class="flex justify-end gap-2">
      <button type="button" class="btn btn-ghost" @click="emit('cancel')">
        Cancel
      </button>
      <button
        type="button"
        class="btn btn-primary"
        :disabled="previewCount === 0"
        @click="handleNext"
      >
        Next →
      </button>
    </div>
  </div>
</template>
