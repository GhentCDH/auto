#!/usr/bin/env bash
# Verify that every #[utoipa::path]-annotated handler in src/api/ is
# registered in the paths() block of src/openapi.rs, and vice versa.

set -euo pipefail

# Build the list of fully-qualified path identifiers that have a
# #[utoipa::path] annotation, e.g. "crate::api::applications::list"
annotated_paths() {
  for file in src/api/*.rs; do
    module=$(basename "$file" .rs)
    if [ "$module" = "mod" ]; then
      prefix="crate::api"
    else
      prefix="crate::api::$module"
    fi
    # For each annotation, find the first "async fn <name>" within 30 lines
    while IFS= read -r lineno; do
      fn_name=$(tail -n +"$lineno" "$file" | head -30 \
        | grep -m1 'async fn ' \
        | sed 's/.*async fn \([a-z_][a-z_0-9]*\).*/\1/')
      [ -n "$fn_name" ] && echo "${prefix}::${fn_name}"
    done < <(grep -n 'utoipa::path' "$file" | cut -d: -f1)
  done | sort -u
}

# Build the list of path identifiers registered in openapi.rs
registered_paths() {
  grep 'crate::api::' src/openapi.rs \
    | grep -o 'crate::api::[a-z_:]*[a-z_]' \
    | sort -u
}

annotated=$(annotated_paths)
registered=$(registered_paths)

missing_reg=$(comm -23 <(echo "$annotated") <(echo "$registered"))
missing_ann=$(comm -13 <(echo "$annotated") <(echo "$registered"))

exit_code=0

if [ -n "$missing_reg" ]; then
  echo "ERROR: annotated with #[utoipa::path] but missing from openapi.rs paths():"
  echo "$missing_reg" | sed 's/^/  /'
  exit_code=1
fi

if [ -n "$missing_ann" ]; then
  echo "WARNING: registered in openapi.rs paths() but no #[utoipa::path] annotation found:"
  echo "$missing_ann" | sed 's/^/  /'
fi

if [ $exit_code -eq 0 ] && [ -z "$missing_ann" ]; then
  echo "OK: openapi.rs is in sync ($(echo "$annotated" | wc -l | tr -d ' ') paths)"
fi

exit $exit_code
