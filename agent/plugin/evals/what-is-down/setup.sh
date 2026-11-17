#!/bin/sh
# Eval runs get a fresh, empty HOME. Point `auto` at a local test server
# (started with auth disabled, see docs/agents.md "Testing the skill"), so no
# token or other secret is needed inside the eval sandbox.
set -eu
url="${EVAL_AUTO_URL:-http://127.0.0.1:18081}"
case "$(uname -s)" in
    Darwin) dir="$HOME/Library/Application Support/auto" ;;
    *) dir="${XDG_CONFIG_HOME:-$HOME/.config}/auto" ;;
esac
mkdir -p "$dir"
printf 'url = "%s"\n' "$url" > "$dir/config.toml"
