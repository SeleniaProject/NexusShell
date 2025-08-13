# Fonts for PNG Rendering

- Place a monospace TTF/OTF here, e.g., `JetBrainsMono-Regular.ttf` or `FiraCode-Regular.ttf`.
- The PNG generator defaults to `assets/fonts/JetBrainsMono-Regular.ttf` if not specified by `--font`.
- Recommended size: 16–22 px for mockups. Adjust with `--size`.
- Multi-language note: ensure the font covers Latin, Japanese, and symbols used in mockups.

Links:
- JetBrainsMono: https://www.jetbrains.com/lp/mono/
- FiraCode: https://github.com/tonsky/FiraCode

Example usage:
```bash
cargo run -p nxsh_ui --bin ansi_to_png -- \
  --font assets/fonts/JetBrainsMono-Regular.ttf \
  --size 18 --bg #282828 \
  --in assets/mockups/nxsh_splash.ans \
  --out assets/mockups/nxsh_splash.png \
  --cols 100 --line-height 1.2
```

