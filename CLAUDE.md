# CLAUDE.md

Rust CLI tool for coupling analysis based on Vlad Khononov's "Balancing Coupling in Software Design".

## Quick Commands

```bash
cargo build                    # Build
cargo test                     # Run tests
cargo clippy -- -D warnings    # Lint
cargo fmt                      # Format
cargo bench                    # Benchmarks

# Run tool
cargo run -- coupling ./src
cargo run -- coupling --summary ./src
cargo run -- coupling --ai ./src
```

## Key Files

- `src/analyzer.rs` - AST analysis with syn
- `src/balance.rs` - Balance score and issue detection
- `src/aposd.rs` - APOSD metrics (module depth, pass-through, cognitive load)
- `src/metrics.rs` - Data structures
- `src/report.rs` - Report generation

## Before Release

Always run before committing:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## References

- Architecture: `.claude/docs/architecture.md`
- Development: `.claude/docs/development.md`
- Commands: `.claude/commands/`
- Agents: `.claude/agents/`
