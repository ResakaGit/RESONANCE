#!/usr/bin/env bash
# Generate an asciicast v2 (.cast) recording from the `demos autopoiesis`
# subcommand without requiring `asciinema` to be installed.
#
# Output is a valid asciinema v2 file ready to:
#   - upload to https://asciinema.org/upload (drag-and-drop, no account needed)
#   - convert to GIF/SVG via:
#       agg input.cast output.gif       (cargo install --git https://github.com/asciinema/agg)
#       svg-term --in input.cast --out output.svg
#
# Format reference: https://docs.asciinema.org/manual/asciicast/v2/
#
# Usage:
#   ./tools/record_demo.sh                       # default: demos autopoiesis
#   OUTPUT=run.cast ./tools/record_demo.sh       # override output path
#   COLS=120 ROWS=40 ./tools/record_demo.sh      # override terminal size

set -euo pipefail

OUTPUT="${OUTPUT:-demo_autopoiesis.cast}"
COLS="${COLS:-100}"
ROWS="${ROWS:-30}"
_DEMOS_BIN="./target/release/demos"
[[ -x "${_DEMOS_BIN}.exe" ]] && _DEMOS_BIN="${_DEMOS_BIN}.exe"
BINARY="${BINARY:-$_DEMOS_BIN}"
SUBCOMMAND="${SUBCOMMAND:-autopoiesis}"
EXTRA_ARGS="${EXTRA_ARGS:---ticks 2000 --sample-every 100}"

# Per-character typing delay (seconds) — realistic shell typing cadence.
TYPE_DELAY="${TYPE_DELAY:-0.04}"
# Inter-line delay while output streams (seconds).
LINE_DELAY="${LINE_DELAY:-0.03}"
# Pause after the prompt (seconds) — gives the viewer a beat to read.
PROMPT_PAUSE="${PROMPT_PAUSE:-0.6}"

if [[ ! -x "$BINARY" ]]; then
  echo "error: $BINARY not built. Run: cargo build --release --bin demos" >&2
  exit 1
fi

# 1. Capture demo output into an array, line by line.
mapfile -t DEMO_LINES < <("$BINARY" "$SUBCOMMAND" $EXTRA_ARGS)

# 2. Emit asciicast v2 header.
TS=$(date +%s)
{
  printf '{"version": 2, "width": %d, "height": %d, "timestamp": %d, ' "$COLS" "$ROWS" "$TS"
  printf '"title": "RESONANCE — autopoiesis demo (cargo run --bin demos -- %s)", ' "$SUBCOMMAND"
  printf '"env": {"SHELL": "/bin/bash", "TERM": "xterm-256color"}}\n'
} > "$OUTPUT"

# JSON string escaper: backslash, quote, newline, tab, control chars.
json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\t'/\\t}"
  s="${s//$'\r'/\\r}"
  printf '%s' "$s"
}

emit_event() {
  local t="$1" data="$2"
  printf '[%.3f, "o", "%s"]\n' "$t" "$(json_escape "$data")" >> "$OUTPUT"
}

# 3. Synthesize prompt + typed command (one event per character).
T=0.0
PROMPT='$ '
emit_event "$T" "$PROMPT"
T=$(awk -v t="$T" -v d="$PROMPT_PAUSE" 'BEGIN{printf "%.3f", t+d}')

CMD="cargo run --release --bin demos -- $SUBCOMMAND $EXTRA_ARGS"
for (( i=0; i<${#CMD}; i++ )); do
  emit_event "$T" "${CMD:$i:1}"
  T=$(awk -v t="$T" -v d="$TYPE_DELAY" 'BEGIN{printf "%.3f", t+d}')
done
emit_event "$T" $'\r\n'
T=$(awk -v t="$T" -v d="$PROMPT_PAUSE" 'BEGIN{printf "%.3f", t+d}')

# 4. Stream demo output line by line with realistic delay.
for line in "${DEMO_LINES[@]}"; do
  emit_event "$T" "${line}"$'\r\n'
  T=$(awk -v t="$T" -v d="$LINE_DELAY" 'BEGIN{printf "%.3f", t+d}')
done

# 5. Final prompt + brief pause (so the cast doesn't end abruptly).
T=$(awk -v t="$T" -v d="0.5" 'BEGIN{printf "%.3f", t+d}')
emit_event "$T" "$PROMPT"

echo "Wrote $OUTPUT ($(wc -l < "$OUTPUT") events, $(stat -c%s "$OUTPUT" 2>/dev/null || stat -f%z "$OUTPUT") bytes)"
echo ""
echo "Next steps:"
echo "  1. Upload  : https://asciinema.org/upload  (drag-and-drop $OUTPUT)"
echo "  2. GIF     : agg $OUTPUT demo.gif      (cargo install --git https://github.com/asciinema/agg)"
echo "  3. SVG     : svg-term --in $OUTPUT --out demo.svg"
echo "  4. Embed   : <script async src=\"https://asciinema.org/a/<ID>.js\" id=\"asciicast-<ID>\"></script>"
