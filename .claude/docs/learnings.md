# Learnings & Design Decisions

## CLI UX

- **Tables fail in CLI**: Box-drawing characters break in many terminals. Use bullet points instead.
- **Strict mode as default**: Showing all issues creates noise. Default to hiding Low severity (60 issues → 3 actionable).
- **Opt-in verbosity**: Use `--all` to show everything, `--verbose` for educational explanations.
- **Multi-language support**: `--japanese`/`--jp` flag for localized explanations, not just translations.

## Khononov Framework

- **3 dimensions are sufficient**: Strength × Distance × Volatility covers all coupling concerns.
- **Balance formula works**: `(STRENGTH XOR DISTANCE) OR NOT VOLATILITY` accurately identifies problems.

## Rust-Specific Patterns

- **Newtype detection**: Single-field tuple structs (`struct UserId(u64)`) indicate good design.
- **Serde derive detection**: `#[derive(Serialize, Deserialize)]` identifies DTOs for separation analysis.
- **Public field exposure**: Direct field access across module boundaries is a code smell.
- **Visibility matters**: `pub(crate)` vs `pub` vs private changes coupling implications.

## Testing Insights

Real-world validation on OSS projects (bat, fd, eza, ripgrep):
- Grade A projects exist (bat: 0.82, fd: 0.83) - tool isn't too strict
- Different architectures show different patterns - tool is sensitive
- Score variance (0.67-0.98) indicates meaningful differentiation

## Docker Best Practices (2025)

- **cargo-chef**: Separates dependency compilation from source, enables layer caching. 5-10x faster rebuilds.
- **distroless/cc-debian12**: 58MB final image. Use `:nonroot` tag for non-root execution.
- **Edition 2024 requires nightly**: `if let` chains and other features need nightly until Rust 1.85+ stabilizes them.
- **ARG scope in multi-stage**: Must redeclare `ARG` after each `FROM` statement.
- **BuildKit cache mounts**: `--mount=type=cache,target=/usr/local/cargo/registry,sharing=locked` persists across builds.
- **Git in distroless**: Not available. Use `Dockerfile.full` (debian-slim) for volatility analysis.

## What Didn't Work

- ~~Connascence types~~: Too granular, removed. Strength levels are sufficient.
- ~~APOSD metrics~~: Overlap with existing analysis, removed for simplicity.
- ~~Temporal coupling~~: Git-based detection was noisy, kept only volatility.
- ~~cargo-chef nightly tag~~: `lukemathwalker/cargo-chef:latest-rust-nightly` doesn't exist. Install nightly manually.
