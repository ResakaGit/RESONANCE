# tools/

One-shot helpers that don't belong in the simulator binaries.

## `record_demo.sh`

Generates an [asciinema](https://asciinema.org) v2 `.cast` recording of
the `demos autopoiesis` subcommand **without requiring asciinema to be
installed**.  It captures real stdout from the binary and synthesizes a
typed-prompt + streamed-output cast with realistic timing.

### Quick start

```bash
cargo build --release --bin demos
./tools/record_demo.sh
# wrote demo_autopoiesis.cast (~3 KB, ~6 seconds duration)
```

### Customize

```bash
OUTPUT=docs/show_hn.cast \
COLS=120 ROWS=40 \
TYPE_DELAY=0.05 LINE_DELAY=0.04 \
EXTRA_ARGS="--ticks 5000 --sample-every 100" \
./tools/record_demo.sh
```

| Env var | Default | Meaning |
|---|---|---|
| `OUTPUT` | `demo_autopoiesis.cast` | output `.cast` path |
| `BINARY` | `./target/release/demos[.exe]` | path to compiled `demos` binary |
| `SUBCOMMAND` | `autopoiesis` | which demo (`autopoiesis` / `papers` / `kleiber`) |
| `EXTRA_ARGS` | `--ticks 2000 --sample-every 100` | passed to the demo |
| `COLS` | `100` | virtual terminal width |
| `ROWS` | `30` | virtual terminal height |
| `TYPE_DELAY` | `0.04` | seconds per typed character of the command |
| `LINE_DELAY` | `0.03` | seconds between output lines |
| `PROMPT_PAUSE` | `0.6` | pause after prompt before typing starts |

### Publish flow (no asciinema install needed)

1. **Upload to asciinema.org** — drag-and-drop your `.cast` at
   <https://asciinema.org/upload>.  Returns a public URL like
   `https://asciinema.org/a/abc123` and an embed snippet.

2. **Convert to GIF / SVG (optional, for HN/blog inline)**:

   ```bash
   # GIF (best quality, animated)
   cargo install --git https://github.com/asciinema/agg
   agg demo_autopoiesis.cast demo.gif

   # SVG (vector, lighter)
   npm install -g svg-term-cli
   svg-term --in demo_autopoiesis.cast --out demo.svg --window
   ```

3. **Embed in HTML / blog**:

   ```html
   <script async src="https://asciinema.org/a/abc123.js" id="asciicast-abc123"></script>
   ```

### Why generate vs record live?

The script generates a cast deterministically from the demo's actual
output — no risk of typos, hesitation, or terminal noise.  Real
asciinema would also work; this just removes the install step and
ensures the recording is reproducible across environments.
