#!/usr/bin/env bash
# Cross-reference BITCOINKERNEL_WARN_UNUSED_RESULT annotations in the upstream
# C header against #[must_use] annotations in the Rust FFI bindings (lib.rs).
#
# Usage: ./check_must_use.sh <header.h> <lib.rs>
# Exit code 0 if in sync, 1 if lib.rs is missing annotations (drift found).
#
# Pure bash + awk, no external dependencies.
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <header.h> <lib.rs>" >&2
    exit 2
fi

HEADER="$1"
RUST="$2"

# --- Step 1: strip /* */ and // comments from the header, char-by-char ---
strip_comments() {
    awk '
    BEGIN { in_block = 0 }
    {
        line = $0
        out = ""
        i = 1
        n = length(line)
        while (i <= n) {
            if (in_block) {
                if (substr(line, i, 2) == "*/") { in_block = 0; i += 2 }
                else { i += 1 }
            } else {
                two = substr(line, i, 2)
                if (two == "/*") { in_block = 1; i += 2 }
                else if (two == "//") { i = n + 1 }
                else { out = out substr(line, i, 1); i += 1 }
            }
        }
        print out
    }' "$1"
}

# --- Step 2: find every "BITCOINKERNEL_API <decl> (" and classify it ---
# Emits: "MUST_USE\t<fn_name>" or "PLAIN\t<fn_name>" for each btck_* declaration.
extract_header_fns() {
    strip_comments "$1" | awk '
    { full = full " " $0 }
    END {
        n = split(full, parts, "BITCOINKERNEL_API")
        for (i = 2; i <= n; i++) {
            p = parts[i]
            paren = index(p, "(")
            if (paren == 0) continue
            decl = substr(p, 1, paren - 1)
            ntok = split(decl, toks, /[ \t]+/)
            fname = ""
            has_must_use = 0
            for (j = 1; j <= ntok; j++) {
                t = toks[j]
                if (t == "") continue
                if (t == "BITCOINKERNEL_WARN_UNUSED_RESULT") has_must_use = 1
                fname = t
            }
            gsub(/^\*+/, "", fname)
            if (fname ~ /^btck_/) {
                print (has_must_use ? "MUST_USE" : "PLAIN") "\t" fname
            }
        }
    }'
}

# --- Step 3: find every "pub fn btck_*(" in lib.rs and whether the
#     immediately preceding non-blank line is "#[must_use]" ---
extract_rust_fns() {
    awk '
    /pub fn btck_[A-Za-z0-9_]*\(/ {
        line = $0
        sub(/^[ \t]*pub fn /, "", line)
        sub(/\(.*/, "", line)
        fname = line
        has_must_use = (prev ~ /#\[must_use\]/) ? 1 : 0
        print (has_must_use ? "MUST_USE" : "PLAIN") "\t" fname "\t" NR
    }
    { if ($0 !~ /^[ \t]*$/) prev = $0 }
    ' "$1"
}

HDR_FNS="$(mktemp)"
RS_FNS="$(mktemp)"
trap 'rm -f "$HDR_FNS" "$RS_FNS"' EXIT

extract_header_fns "$HEADER" > "$HDR_FNS"
extract_rust_fns "$RUST" > "$RS_FNS"

hdr_total=$(wc -l < "$HDR_FNS")
hdr_must_use=$(grep -c '^MUST_USE' "$HDR_FNS" || true)
rust_total=$(wc -l < "$RS_FNS")

echo "Header functions total:            $hdr_total"
echo "Header functions marked must-use:   $hdr_must_use"
echo "Rust functions found:                $rust_total"
echo

missing=0
not_in_rust=0

# not_in_rust: header must-use fns absent from lib.rs
: > /tmp/_not_in_rust.$$
grep '^MUST_USE' "$HDR_FNS" | cut -f2 | while read -r fn; do
    if ! awk -F'\t' -v f="$fn" '$2==f{found=1} END{exit !found}' "$RS_FNS"; then
        echo "$fn" >> /tmp/_not_in_rust.$$
    fi
done
if [[ -s /tmp/_not_in_rust.$$ ]]; then
    n=$(wc -l < /tmp/_not_in_rust.$$)
    echo "WARNING: in header as must-use but NOT FOUND in lib.rs ($n):"
    sed 's/^/   - /' /tmp/_not_in_rust.$$
    echo
fi
rm -f /tmp/_not_in_rust.$$

# missing: header must-use fns present in lib.rs but not marked
: > /tmp/_missing.$$
grep '^MUST_USE' "$HDR_FNS" | cut -f2 | while read -r fn; do
    line=$(awk -F'\t' -v f="$fn" '$2==f{print $1"\t"$3}' "$RS_FNS")
    if [[ -n "$line" ]]; then
        status="${line%%$'\t'*}"
        lineno="${line##*$'\t'}"
        if [[ "$status" != "MUST_USE" ]]; then
            printf '%s\t%s\n' "$lineno" "$fn" >> /tmp/_missing.$$
        fi
    fi
done
if [[ -s /tmp/_missing.$$ ]]; then
    n=$(wc -l < /tmp/_missing.$$)
    echo "MISSING #[must_use] in lib.rs ($n):"
    sort -n /tmp/_missing.$$ | awk -F'\t' '{printf "   line %4d: %s\n", $1, $2}'
    echo
    missing=$n
else
    echo "OK: no missing #[must_use] annotations."
    echo
fi
rm -f /tmp/_missing.$$

# extra: rust fns marked must_use but header doesn't mark them
: > /tmp/_extra.$$
awk -F'\t' '$1=="MUST_USE"{print $2"\t"$3}' "$RS_FNS" | while IFS=$'\t' read -r fn lineno; do
    if ! awk -F'\t' -v f="$fn" '$1=="MUST_USE" && $2==f{found=1} END{exit !found}' "$HDR_FNS"; then
        printf '%s\t%s\n' "$lineno" "$fn" >> /tmp/_extra.$$
    fi
done
if [[ -s /tmp/_extra.$$ ]]; then
    n=$(wc -l < /tmp/_extra.$$)
    echo "INFO: #[must_use] in lib.rs but NOT marked in header ($n) - double check these:"
    sort -n /tmp/_extra.$$ | awk -F'\t' '{printf "   line %4d: %s\n", $1, $2}'
    echo
fi
rm -f /tmp/_extra.$$

correct=$(awk -F'\t' '$1=="MUST_USE"{print $2}' "$RS_FNS" | sort > /tmp/_rmu.$$; \
          awk -F'\t' '$1=="MUST_USE"{print $2}' "$HDR_FNS" | sort > /tmp/_hmu.$$; \
          comm -12 /tmp/_rmu.$$ /tmp/_hmu.$$ | wc -l)
rm -f /tmp/_rmu.$$ /tmp/_hmu.$$
echo "Already correct: $correct"

if [[ $missing -gt 0 ]]; then
    exit 1
fi
exit 0
