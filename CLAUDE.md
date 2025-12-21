# cargo-coupling

Rust CLI for coupling analysis based on Khononov's "Balancing Coupling in Software Design".

## Commands

```bash
# Development
cargo build && cargo test && cargo fmt --all && cargo clippy -- -D warnings

# Run
cargo run -- coupling ./src          # Analyze
cargo run -- coupling --web ./src    # Web UI (:3000)

# Docker
docker build -t cargo-coupling .
docker run --rm -v $(pwd):/workspace cargo-coupling coupling /workspace/src
docker compose up web

# Release
cargo fmt --all && cargo clippy -- -D warnings && cargo test
git tag -a vX.Y.Z -m "Release vX.Y.Z" && git push origin vX.Y.Z
# → GitHub Actions auto-publishes to crates.io & ghcr.io
```

## Key Files

| Path | Purpose |
|------|---------|
| `src/analyzer.rs` | AST analysis (syn) |
| `src/balance.rs` | Balance score calculation |
| `src/web/` | Web visualization server |
| `Dockerfile` | distroless (58MB) |
| `Dockerfile.full` | debian-slim + Git |

## Docs & Rules

- `.claude/docs/` - Khononov framework, issue types, learnings
- `.claude/rules/` - Rust, Web UI rules
- `.claude/skills/` - analyze, release skills
- `.claude/commands/` - Slash commands

## Notes

**Edition 2024**: `if let` chains require nightly (< Rust 1.85)

**Docker**:
- cargo-chef: 依存キャッシュで 5-10x 高速化
- distroless: 非root、CVE最小
- ARG: `FROM` 後に再宣言必要
- Git なし → `Dockerfile.full` 使用

**ghcr.io push**:
```bash
gh auth refresh -h github.com -s write:packages
gh auth token | docker login ghcr.io -u nwiizo --password-stdin
```

**Docker 実行**: `cargo-coupling coupling ...` (not `cargo coupling`)
