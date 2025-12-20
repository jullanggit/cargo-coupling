# Refactor Skill

Concrete refactoring proposals based on analysis.

## Issue Types

| Type | Description |
|------|-------------|
| `global-complexity` | Strong coupling to distant modules |
| `cascading-change` | Coupling to volatile modules |
| `inappropriate-intimacy` | Internal access across boundaries |
| `high-efferent` | Too many outgoing dependencies |
| `high-afferent` | Too many incoming dependencies |

## Common Patterns

### Global Complexity Fix

```rust
// Before: Direct dependency on distant module
use crate::deep::nested::InternalType;

// After: Trait abstraction
use crate::traits::Processable;
fn process(p: &impl Processable) { ... }
```

### High Efferent Fix

```rust
// Before: Many dependencies
use crate::a::A;
use crate::b::B;
use crate::c::C;
// ... 15+ imports

// After: Facade pattern
use crate::facade::ServiceFacade;
```

## Verification

```bash
# Before refactoring
cargo run -- coupling --summary ./src > before.txt

# After refactoring
cargo run -- coupling --summary ./src > after.txt

# Compare
diff before.txt after.txt
```

## Guidelines

- Split large changes into small commits
- Verify tests pass at each step
- Document change intent for reviewers
