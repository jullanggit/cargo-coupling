# Development Guide

## Common Tasks

### Adding a New Issue Type

1. Add variant to `IssueType` enum in `balance.rs`
2. Implement detection in `identify_coupling_issues()`
3. Add description in `IssueType::description()`
4. Add suggested action in `suggest_refactoring()`

### Adding a CLI Option

1. Add field to `Args` struct in `main.rs`
2. Use the value in `run()` function
3. Update README.md CLI section

### Modifying Strength Detection

1. Update `UsageContext` enum in `analyzer.rs`
2. Add/modify visitor in `CouplingAnalyzer` impl
3. Update `UsageContext::to_strength()` mapping

## False Positive Filtering

The analyzer filters out:
- `Self::Self` patterns
- Short lowercase names (likely local variables)
- Duplicate patterns like `foo::foo`
- Common local variable names
- Primitive and std types (Option, Result, Vec, etc.)

## Performance Optimization

### Git Analysis (`volatility.rs`)

```rust
Command::new("git")
    .args([
        "log", "--pretty=format:", "--name-only",
        "--diff-filter=AMRC",  // Skip deleted files
        &format!("--since={} months ago", months),
        "--", "*.rs",  // Filter at Git level
    ])
```

Techniques:
1. Git-level path filtering (`-- "*.rs"`)
2. Skip deleted files (`--diff-filter=AMRC`)
3. 64KB buffer for streaming
4. `spawn()` for immediate processing

### Parallel Processing

```rust
file_paths.par_iter()
    .filter_map(|path| analyze_rust_file_full(path).ok())
    .collect()
```

Control with `-j N` option.

## Benchmarks

| Project | Files | With Git | Without Git |
|---------|-------|----------|-------------|
| tokio | 488 | 655ms | 234ms |
| alacritty | 83 | 298ms | 161ms |
| ripgrep | 59 | 181ms | - |
