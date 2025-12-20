# Explain Issue Skill

Detailed explanations of coupling issue types.

## Issue Types

### Global Complexity

- **Condition**: Strong coupling + far distance
- **Formula**: `STRENGTH >= 0.5 AND DISTANCE >= 0.5`
- **Fix**: Introduce traits, move modules, facade pattern

### Cascading Change Risk

- **Condition**: Strong coupling + high volatility
- **Formula**: `STRENGTH >= 0.5 AND VOLATILITY >= 0.5`
- **Fix**: Stable interface layer, dependency inversion

### Inappropriate Intimacy

- **Condition**: Intrusive coupling across boundaries
- **Formula**: `STRENGTH = 1.0 AND DISTANCE > 0.0`
- **Fix**: Encapsulation, `pub(crate)` usage

### High Efferent Coupling

- **Condition**: Module has too many outgoing dependencies
- **Threshold**: Default 20
- **Fix**: Module splitting, facade pattern

### High Afferent Coupling

- **Condition**: Module has too many incoming dependencies
- **Threshold**: Default 30
- **Fix**: Interface introduction, responsibility distribution

### Unnecessary Abstraction

- **Condition**: Weak coupling + near distance + low volatility
- **Formula**: `STRENGTH < 0.3 AND DISTANCE < 0.3 AND VOLATILITY < 0.3`
- **Fix**: Remove abstraction, use direct implementation

## Related Principles

- **SOLID**: Single Responsibility, Dependency Inversion
- **Khononov Balance**: `(STRENGTH XOR DISTANCE) OR NOT VOLATILITY`
- **Rust idioms**: Trait-based abstraction, module visibility

## Reference

- [Balancing Coupling in Software Design](https://www.amazon.com/dp/B0FVDYKJYQ)
