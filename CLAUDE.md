# CLAUDE.md

Rust CLI for coupling analysis based on Khononov's "Balancing Coupling in Software Design".

## Quick Commands

```bash
cargo build                    # Build
cargo test                     # Test
cargo fmt --all                # Format
cargo clippy -- -D warnings    # Lint
cargo run -- coupling ./src    # Analyze
cargo run -- coupling --web ./src  # Web UI
```

## Docker

```bash
docker build -t cargo-coupling .                    # Build
docker run --rm -v $(pwd):/workspace cargo-coupling coupling /workspace/src
docker compose up web                               # Web UI on :3000
```

**Note**: Edition 2024 requires nightly Rust. Dockerfile uses `rustup default nightly`.

## Key Files

| Path | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis with syn |
| `src/balance.rs` | Balance score calculation |
| `src/web/` | Web visualization server |
| `web-assets/` | Frontend (HTML/CSS/JS) |
| `Dockerfile` | distroless build (58MB) |
| `Dockerfile.full` | Git-enabled build |

## Configuration

```toml
# .coupling.toml
[thresholds]
max_efferent_coupling = 15
max_afferent_coupling = 20
```

## Documentation

| Topic | Location |
|-------|----------|
| Khononov Framework | `.claude/docs/khononov-framework.md` |
| Issue Types | `.claude/docs/issue-types.md` |
| Web UI Architecture | `.claude/docs/web-ui-architecture.md` |
| Design Decisions | `.claude/docs/learnings.md` |
| JTBD | `.claude/docs/jobs-to-be-done.md` |

## Rules & Skills

| Type | Location |
|------|----------|
| Rust Rules | `.claude/rules/rust.md` |
| Web UI Rules | `.claude/rules/web-ui.md` |
| Analysis Skill | `.claude/skills/analyze/SKILL.md` |
| Release Skill | `.claude/skills/release/SKILL.md` |

## Commands

See `.claude/commands/` for available slash commands.

## Learnings

### Release Workflow

```bash
cargo fmt --all && cargo clippy -- -D warnings && cargo test
git tag -a v0.2.5 -m "Release v0.2.5"
git push origin v0.2.5
# GitHub Actions auto-publishes to crates.io
```

### Docker (2025 Best Practices)

- **cargo-chef**: 依存関係を分離してキャッシュ。リビルド 5-10x 高速化
- **distroless/cc-debian12:nonroot**: 58MB、非root実行、CVE最小
- **Edition 2024**: `if let` chains 等は nightly 必須（Rust 1.85 未満）
- **ARG scope**: `FROM` 後に再宣言必要（マルチステージビルド時）
- **cargo-chef nightly**: 公式タグなし → `rustup default nightly` で手動設定
- **Git in distroless**: 含まれない → `Dockerfile.full` (debian-slim) を使用

### ghcr.io Push

```bash
gh auth refresh -h github.com -s write:packages  # 権限追加
gh auth token | docker login ghcr.io -u USER --password-stdin
docker push ghcr.io/USER/IMAGE:TAG
```

### cargo subcommand

Docker で実行時は `cargo-coupling coupling ...` の形式で呼び出す（`cargo coupling` ではない）。
