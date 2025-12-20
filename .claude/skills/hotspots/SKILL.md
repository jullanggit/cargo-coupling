# Hotspots Skill

Identify modules needing refactoring attention.

## Command

```bash
# Show top 5 hotspots (default)
cargo run -- coupling --hotspots ./src

# Show top 10 hotspots
cargo run -- coupling --hotspots=10 ./src

# With explanations
cargo run -- coupling --hotspots --verbose ./src
```

## Scoring Factors

| Factor | Weight | Description |
|--------|--------|-------------|
| Issue count | ×30 | Related coupling issues |
| Coupling count | ×5 | In/out dependencies |
| Health: Critical | +50 | Critical health status |
| Health: Needs Review | +20 | Review needed |
| Circular dependency | +40 | Part of a cycle |

## Analysis Focus

1. **Why problematic**
   - Issue types detected
   - Coupling patterns

2. **Priority**
   - Blast radius
   - Difficulty
   - Expected benefit

3. **Refactoring suggestions**
   - Interface separation
   - Dependency inversion
   - Module splitting
   - Facade pattern

## Notes

- Score 0 = no issues
- Circular dependencies = highest priority
- Focus on top 5 for actionable results
