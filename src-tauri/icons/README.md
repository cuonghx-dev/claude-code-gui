# Icons

Placeholder. Replace before 1.0 release. Required files:

- `32x32.png`
- `128x128.png`
- `128x128@2x.png` (macOS retina)
- `icon.icns` (macOS bundle)
- `icon.ico` (Windows bundle)

Generate with [`tauri icon`](https://tauri.app/develop/icons/) from a single 1024x1024 master:

```bash
cargo tauri icon path/to/master.png
```
