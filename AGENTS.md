# AGENTS.md

- Use a 10s delay before each Azure/OpenAI request to avoid Azure limitation too many requests
- Always use Pull Request to work on
- Commit and push when finish working. Comment PR with changes made. Update PR when some feature is add or change.
- Run tests to ensure nothing break
- Run chart_player_rust to ensure no errors

## Repository / Clone

```bash
https://github.com/goguitar/ChartPlayer
```

Submodules: `Dependencies/UILayout`, `Dependencies/OpenSongChart`, `Dependencies/PitchDetect`.


## Build (Rust)

```bash
cd chart_player_rust
cargo build

# Run image processor
cargo run --bin image_processor

# Run app
cargo run --bin chart_player
```

## Runtime Dependencies

- **.NET 8.0** required for VST plugin
- **Rust**
- **WGPU** required for Game 3D scene 
- **CPAL** required for Rust audio
- **Jack Audio** required for ChartPlayerJack on Linux/Mac
- **librubberband2** (Rubber Band library) for pitch shifting on Linux

## Project Structure

| Directory | Description |
|-----------|--------------|
| `ChartPlayer/` | Main Windows app (C#) |
| `ChartPlayerPlugin/` | VST3 plugin |
| `ChartPlayerJack/` | Jack Audio client (Linux/Mac) |
| `chart_player_rust/` | Experimental Rust rewrite |
| `ImageProcessor/` | Texture asset generator |
| `Dependencies/` | Submodules (UILayout, OpenSongChart, PitchDetect) |

Convert all subprojects to Rust, keep filenames and directory structure

## CI Build Order

See `.github/workflows/build.yml` for the authoritative build sequence used in CI.
