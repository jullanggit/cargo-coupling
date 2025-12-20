# Web Visualization Skill

Interactive web UI for coupling analysis.

## Quick Start

```bash
# Start web server
cargo run -- coupling --web ./src

# Custom port
cargo run -- coupling --web --port 8080 ./src

# Don't auto-open browser
cargo run -- coupling --web --no-open ./src
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` | Focus search |
| `f` | Fit to screen |
| `r` | Reset layout |
| `e` | Export PNG |
| `Esc` | Clear selection |
| `?` | Show help |

## Panel Features

### Hotspots
Top refactoring targets ranked by issue severity.

### Key Modules
- Connections: Sort by dependency count
- Issues: Sort by problem count
- Health: Sort by health score

### Analysis
- Show Dependents: Modules depending on selected
- Show Dependencies: Modules selected depends on
- Full Impact: Complete blast radius

### Filters
- Strength: Intrusive/Functional/Model/Contract
- Distance: Same/Different module
- Volatility: High/Medium/Low
- Balance Score: Range filter
- Show Issues Only / Show Cycles Only

## Graph Interaction

- **Click node**: Highlight neighbors, center view
- **Click edge**: Show dependency direction
- **Click background**: Clear selection
