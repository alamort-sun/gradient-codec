#!/usr/bin/env bash
# A5 plane-deny (Susano dry-audit v13 / Kaliseph A5 tertiary rule)
#
# One deny list. Greps plane-facing Rust for banned field declarations.
# Exit 1 on any hit.
#
#   ./scripts/ci/a5-plane-deny.sh
#   ./scripts/ci/a5-plane-deny.sh ../gradient-space-time/crates
#   ./scripts/ci/a5-plane-deny.sh src vecGradient ../gradient-space-time/crates
#
# Canonical home: gradient-codec (plane-boundary law). Space-time CI vendors
# the same rules so leaf PRs stay gated without alamort gitlink CI.

set -euo pipefail

DENY_FIELDS=(trace_id trajectory_id prompt_hash generated_text)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEC_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

roots=()
if [[ $# -gt 0 ]]; then
  for r in "$@"; do
    roots+=("${r}")
  done
else
  # Plane prelude surface (not the whole router bin): plane seal + geometry
  # crate + sibling space-time tables when present in monorepo layout.
  # Ephemeral router modules (trace/routing/adapter) are out of scope — prompt_hash
  # there is still wrong long-term, but A5 gates the plane, not the orchestration bin.
  if [[ -f "${CODEC_ROOT}/src/plane.rs" ]]; then
    roots+=("${CODEC_ROOT}/src/plane.rs")
  fi
  if [[ -d "${CODEC_ROOT}/src/plane" ]]; then
    roots+=("${CODEC_ROOT}/src/plane")
  fi
  if [[ -d "${CODEC_ROOT}/vecGradient" ]]; then
    roots+=("${CODEC_ROOT}/vecGradient")
  fi
  sibling="${CODEC_ROOT}/../gradient-space-time/crates"
  if [[ -d "${sibling}" ]]; then
    roots+=("${sibling}")
  fi
fi

# Resolve to existing paths only
existing=()
for r in "${roots[@]}"; do
  if [[ -e "${r}" ]]; then
    existing+=("${r}")
  else
    echo "a5-plane-deny: skip missing root: ${r}" >&2
  fi
done
if [[ ${#existing[@]} -eq 0 ]]; then
  echo "a5-plane-deny: no roots to scan" >&2
  exit 2
fi

echo "a5-plane-deny: deny fields = ${DENY_FIELDS[*]} (+ ProviderId in durable/plane paths)"
echo "a5-plane-deny: roots = ${existing[*]}"

# Prefer rg; fall back to grep -R.
if command -v rg >/dev/null 2>&1; then
  search() {
    local pat="$1"; shift
    rg -n --no-heading -g '*.rs' -g '!boundary.rs' --pcre2 "${pat}" "$@" || true
  }
else
  search() {
    local pat="$1"; shift
    grep -RIn --include='*.rs' --exclude='boundary.rs' -E "${pat}" "$@" || true
  }
fi

hits=()

# Field decl: start-of-line (after space) optional pub/mut, name, colon.
# Negative lookbehind for word char so original_trace_id does not match trace_id
# when using PCRE; grep -E fallback uses (^|[^A-Za-z0-9_]) anchor.
for name in "${DENY_FIELDS[@]}"; do
  if command -v rg >/dev/null 2>&1; then
    pat="(?<!\\w)(pub\\s+)?(mut\\s+)?${name}\\s*:"
  else
    pat="(^|[^A-Za-z0-9_])(pub[[:space:]]+)?(mut[[:space:]]+)?${name}[[:space:]]*:"
  fi
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    # Drop comment-only lines (path:lineno:content)
    content="${line#*:}"
    content="${content#*:}"
    case "${content}" in
      [[:space:]]*'//'*) continue ;;
    esac
    hits+=("${line}  [banned field ${name}]")
  done < <(search "${pat}" "${existing[@]}")
done

# ProviderId as durable/plane column type — only tables.rs / types.rs / spacetime-module
provider_roots=()
for r in "${existing[@]}"; do
  if [[ -d "${r}" ]]; then
    while IFS= read -r -d '' f; do
      case "${f}" in
        */tables.rs|*/types.rs|*/spacetime-module/*.rs|*/spacetime-module/*/*.rs)
          provider_roots+=("${f}")
          ;;
      esac
    done < <(find "${r}" -type f \( -name 'tables.rs' -o -name 'types.rs' -o -path '*/spacetime-module/*.rs' \) -print0 2>/dev/null)
  elif [[ -f "${r}" ]]; then
    case "${r}" in
      */tables.rs|*/types.rs|*/spacetime-module/*) provider_roots+=("${r}") ;;
    esac
  fi
done

if [[ ${#provider_roots[@]} -gt 0 ]]; then
  if command -v rg >/dev/null 2>&1; then
    ppat=':\\s*(Option\\s*<\\s*)?ProviderId\\b|:\\s*Vec\\s*<\\s*ProviderId\\b'
  else
    ppat=':[[:space:]]*(Option[[:space:]]*<[[:space:]]*)?ProviderId|:[[:space:]]*Vec[[:space:]]*<[[:space:]]*ProviderId'
  fi
  while IFS= read -r line; do
    [[ -z "${line}" ]] && continue
    content="${line#*:}"
    content="${content#*:}"
    case "${content}" in
      [[:space:]]*'//'*) continue ;;
    esac
    hits+=("${line}  [ProviderId in durable/plane type]")
  done < <(search "${ppat}" "${provider_roots[@]}")
fi

if [[ ${#hits[@]} -gt 0 ]]; then
  echo "a5-plane-deny: FAIL — ${#hits[@]} hit(s):" >&2
  printf '  %s\n' "${hits[@]}" >&2
  exit 1
fi

echo "a5-plane-deny: OK"
