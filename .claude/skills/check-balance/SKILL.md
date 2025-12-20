# Check Balance Skill

Quick health check for coupling balance.

## Command

```bash
cargo run -- coupling --summary ./src
```

## Health Grades

| Grade | Score | Status |
|-------|-------|--------|
| A | 0.90-1.00 | Excellent |
| B | 0.80-0.89 | Good |
| C | 0.60-0.79 | Acceptable |
| D | 0.40-0.59 | Needs improvement |
| F | 0.00-0.39 | Critical |

## Issue Severity

| Severity | Action |
|----------|--------|
| Critical | Fix immediately |
| High | Fix within 1 week |
| Medium | Plan to fix |
| Low | Monitor |

## Quick Checks

```bash
# Summary only
cargo run -- coupling --summary ./src

# Japanese output
cargo run -- coupling --summary --japanese ./src

# CI/CD quality gate
cargo run -- coupling --check --min-grade=C ./src
```

## Next Steps

- Low score: Run `/analyze` for details
- Critical issues: Run `/full-review` for comprehensive analysis
