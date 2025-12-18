//! Connascence type detection based on Meilir Page-Jones' taxonomy
//!
//! Connascence is a software quality metric that describes the degree to which
//! changes in one component require changes in another. This module provides
//! detection for various connascence types through static analysis.
//!
//! ## Connascence Types (from weakest to strongest)
//!
//! ### Static Connascence (detectable at compile time)
//!
//! 1. **Name** - Components must agree on the name of something
//! 2. **Type** - Components must agree on the type of something
//! 3. **Meaning** - Components must agree on the meaning of particular values
//! 4. **Position** - Components must agree on the order of elements
//! 5. **Algorithm** - Components must agree on a particular algorithm
//!
//! ### Dynamic Connascence (only visible at runtime)
//!
//! 6. **Execution** - Components must be executed in a particular order
//! 7. **Timing** - Components must be timed in relation to each other
//! 8. **Value** - Components must agree on specific values at runtime
//! 9. **Identity** - Components must reference the same object
//!
//! ## References
//!
//! - Meilir Page-Jones, "What Every Programmer Should Know About OOD"
//! - Jim Weirich, "Grand Unified Theory of Software Design" (talk)

use std::collections::HashMap;

/// Types of connascence that can be detected through static analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnascenceType {
    /// Connascence of Name - Agreement on names
    ///
    /// Example: Using a function/type by its name
    /// Strength: Weakest (easy to refactor with rename)
    Name,

    /// Connascence of Type - Agreement on types
    ///
    /// Example: Function parameter types, struct field types
    /// Strength: Weak-Medium
    Type,

    /// Connascence of Meaning - Agreement on semantic values
    ///
    /// Example: Magic numbers, special string values
    /// Strength: Medium
    Meaning,

    /// Connascence of Position - Agreement on ordering
    ///
    /// Example: Function argument order, tuple element order
    /// Strength: Medium-Strong
    Position,

    /// Connascence of Algorithm - Agreement on algorithms
    ///
    /// Example: Encoding/decoding pairs, hash functions
    /// Strength: Strong
    Algorithm,
}

impl ConnascenceType {
    /// Get the strength value (0.0 - 1.0, higher = stronger coupling)
    pub fn strength(&self) -> f64 {
        match self {
            ConnascenceType::Name => 0.2,
            ConnascenceType::Type => 0.4,
            ConnascenceType::Meaning => 0.6,
            ConnascenceType::Position => 0.7,
            ConnascenceType::Algorithm => 0.9,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ConnascenceType::Name => "Agreement on names (renaming affects both)",
            ConnascenceType::Type => "Agreement on types (type changes affect both)",
            ConnascenceType::Meaning => "Agreement on semantic values (magic values)",
            ConnascenceType::Position => "Agreement on ordering (positional coupling)",
            ConnascenceType::Algorithm => "Agreement on algorithm (algorithm changes affect both)",
        }
    }

    /// Get refactoring suggestion
    pub fn refactoring_suggestion(&self) -> &'static str {
        match self {
            ConnascenceType::Name => "Use IDE rename refactoring to change safely",
            ConnascenceType::Type => "Consider using traits/generics to reduce type coupling",
            ConnascenceType::Meaning => "Replace magic values with named constants or enums",
            ConnascenceType::Position => "Use named parameters or builder pattern",
            ConnascenceType::Algorithm => {
                "Extract algorithm into shared module with clear contract"
            }
        }
    }
}

/// Detected connascence instance
#[derive(Debug, Clone)]
pub struct ConnascenceInstance {
    /// Type of connascence
    pub connascence_type: ConnascenceType,
    /// Source location (module/file)
    pub source: String,
    /// Target location (module/file/item)
    pub target: String,
    /// Additional context (e.g., which name, which type)
    pub context: String,
    /// Line number if available
    pub line: Option<usize>,
}

impl ConnascenceInstance {
    pub fn new(
        connascence_type: ConnascenceType,
        source: String,
        target: String,
        context: String,
    ) -> Self {
        Self {
            connascence_type,
            source,
            target,
            context,
            line: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

/// Statistics about connascence types in a project
#[derive(Debug, Clone, Default)]
pub struct ConnascenceStats {
    /// Count by type
    pub by_type: HashMap<ConnascenceType, usize>,
    /// Total instances
    pub total: usize,
    /// Weighted strength score
    pub weighted_strength: f64,
}

impl ConnascenceStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a connascence instance
    pub fn add(&mut self, connascence_type: ConnascenceType) {
        *self.by_type.entry(connascence_type).or_insert(0) += 1;
        self.total += 1;
        self.weighted_strength += connascence_type.strength();
    }

    /// Get average strength (0.0 - 1.0)
    pub fn average_strength(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.weighted_strength / self.total as f64
        }
    }

    /// Get count for a specific type
    pub fn count(&self, connascence_type: ConnascenceType) -> usize {
        self.by_type.get(&connascence_type).copied().unwrap_or(0)
    }

    /// Get percentage for a specific type
    pub fn percentage(&self, connascence_type: ConnascenceType) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.count(connascence_type) as f64 / self.total as f64) * 100.0
        }
    }
}

/// Analyzer for detecting connascence patterns
#[derive(Debug, Default, Clone)]
pub struct ConnascenceAnalyzer {
    /// Detected instances
    pub instances: Vec<ConnascenceInstance>,
    /// Statistics
    pub stats: ConnascenceStats,
    /// Current module being analyzed
    current_module: String,
    /// Function signatures for position analysis (fn_name -> arg_count)
    function_signatures: HashMap<String, usize>,
    /// Magic number patterns detected
    magic_numbers: Vec<(String, String)>, // (location, value)
}

impl ConnascenceAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set current module context
    pub fn set_module(&mut self, module: String) {
        self.current_module = module;
    }

    /// Record a name dependency (Connascence of Name)
    pub fn record_name_dependency(&mut self, target: &str, context: &str) {
        let instance = ConnascenceInstance::new(
            ConnascenceType::Name,
            self.current_module.clone(),
            target.to_string(),
            context.to_string(),
        );
        self.instances.push(instance);
        self.stats.add(ConnascenceType::Name);
    }

    /// Record a type dependency (Connascence of Type)
    pub fn record_type_dependency(&mut self, type_name: &str, usage_context: &str) {
        let instance = ConnascenceInstance::new(
            ConnascenceType::Type,
            self.current_module.clone(),
            type_name.to_string(),
            usage_context.to_string(),
        );
        self.instances.push(instance);
        self.stats.add(ConnascenceType::Type);
    }

    /// Record a positional dependency (Connascence of Position)
    ///
    /// This is detected when a function has many positional arguments
    pub fn record_position_dependency(&mut self, fn_name: &str, arg_count: usize) {
        // Only flag as positional coupling if there are 4+ arguments
        if arg_count >= 4 {
            let instance = ConnascenceInstance::new(
                ConnascenceType::Position,
                self.current_module.clone(),
                fn_name.to_string(),
                format!("Function with {} positional arguments", arg_count),
            );
            self.instances.push(instance);
            self.stats.add(ConnascenceType::Position);
        }
        self.function_signatures
            .insert(fn_name.to_string(), arg_count);
    }

    /// Record a magic number (Connascence of Meaning)
    pub fn record_magic_number(&mut self, location: &str, value: &str) {
        // Skip common acceptable values
        if is_acceptable_literal(value) {
            return;
        }

        let instance = ConnascenceInstance::new(
            ConnascenceType::Meaning,
            self.current_module.clone(),
            location.to_string(),
            format!("Magic value: {}", value),
        );
        self.instances.push(instance);
        self.stats.add(ConnascenceType::Meaning);
        self.magic_numbers
            .push((location.to_string(), value.to_string()));
    }

    /// Record an algorithm dependency (Connascence of Algorithm)
    ///
    /// This is detected heuristically for known patterns like:
    /// - encode/decode pairs
    /// - serialize/deserialize
    /// - hash functions used in multiple places
    pub fn record_algorithm_dependency(&mut self, pattern: &str, context: &str) {
        let instance = ConnascenceInstance::new(
            ConnascenceType::Algorithm,
            self.current_module.clone(),
            pattern.to_string(),
            context.to_string(),
        );
        self.instances.push(instance);
        self.stats.add(ConnascenceType::Algorithm);
    }

    /// Get summary report
    pub fn summary(&self) -> String {
        let mut report = String::new();
        report.push_str("## Connascence Analysis\n\n");
        report.push_str(&format!("**Total Instances**: {}\n", self.stats.total));
        report.push_str(&format!(
            "**Average Strength**: {:.2}\n\n",
            self.stats.average_strength()
        ));

        report.push_str("| Type | Count | % | Strength | Description |\n");
        report.push_str("|------|-------|---|----------|-------------|\n");

        for conn_type in [
            ConnascenceType::Name,
            ConnascenceType::Type,
            ConnascenceType::Meaning,
            ConnascenceType::Position,
            ConnascenceType::Algorithm,
        ] {
            let count = self.stats.count(conn_type);
            if count > 0 {
                report.push_str(&format!(
                    "| {:?} | {} | {:.1}% | {:.1} | {} |\n",
                    conn_type,
                    count,
                    self.stats.percentage(conn_type),
                    conn_type.strength(),
                    conn_type.description()
                ));
            }
        }

        report
    }

    /// Get high-strength instances that should be reviewed
    pub fn high_strength_instances(&self) -> Vec<&ConnascenceInstance> {
        self.instances
            .iter()
            .filter(|i| i.connascence_type.strength() >= 0.6)
            .collect()
    }
}

/// Check if a literal value is acceptable (not a magic number)
fn is_acceptable_literal(value: &str) -> bool {
    // Common acceptable numeric values
    let acceptable_numbers = [
        "0", "1", "2", "-1", "0.0", "1.0", "0.5", "100", "1000", "true", "false",
    ];

    if acceptable_numbers.contains(&value) {
        return true;
    }

    // Check for common patterns
    if value.starts_with('"') || value.starts_with('\'') {
        // String literals - check for common acceptable patterns
        let inner = value.trim_matches(|c| c == '"' || c == '\'');
        // Empty string, single char, common separators
        return inner.is_empty()
            || inner.len() == 1
            || inner == " "
            || inner == "\n"
            || inner == ","
            || inner == ":"
            || inner == "/"
            || inner.starts_with("http")
            || inner.starts_with("https");
    }

    false
}

/// Detect potential algorithm connascence patterns in code
pub fn detect_algorithm_patterns(content: &str) -> Vec<(&'static str, String)> {
    let mut patterns = Vec::new();

    // Check for encode/decode pairs
    if content.contains("encode") && content.contains("decode") {
        patterns.push(("encode/decode", "Encoding algorithm must match".to_string()));
    }

    // Check for serialize/deserialize
    if content.contains("serialize") && content.contains("deserialize") {
        patterns.push((
            "serialize/deserialize",
            "Serialization format must match".to_string(),
        ));
    }

    // Check for hash patterns
    if (content.contains("hash") || content.contains("Hash"))
        && (content.contains("sha") || content.contains("md5") || content.contains("blake"))
    {
        patterns.push((
            "hash algorithm",
            "Hash algorithm must be consistent".to_string(),
        ));
    }

    // Check for compression patterns
    if content.contains("compress") && content.contains("decompress") {
        patterns.push((
            "compression",
            "Compression algorithm must match".to_string(),
        ));
    }

    // Check for encryption patterns
    if content.contains("encrypt") && content.contains("decrypt") {
        patterns.push(("encryption", "Encryption algorithm must match".to_string()));
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connascence_type_strength() {
        assert!(ConnascenceType::Name.strength() < ConnascenceType::Type.strength());
        assert!(ConnascenceType::Type.strength() < ConnascenceType::Meaning.strength());
        assert!(ConnascenceType::Position.strength() < ConnascenceType::Algorithm.strength());
    }

    #[test]
    fn test_connascence_stats() {
        let mut stats = ConnascenceStats::new();
        stats.add(ConnascenceType::Name);
        stats.add(ConnascenceType::Name);
        stats.add(ConnascenceType::Type);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.count(ConnascenceType::Name), 2);
        assert_eq!(stats.count(ConnascenceType::Type), 1);
    }

    #[test]
    fn test_analyzer_name_dependency() {
        let mut analyzer = ConnascenceAnalyzer::new();
        analyzer.set_module("test_module".to_string());
        analyzer.record_name_dependency("SomeType", "use statement");

        assert_eq!(analyzer.instances.len(), 1);
        assert_eq!(analyzer.stats.count(ConnascenceType::Name), 1);
    }

    #[test]
    fn test_position_dependency_threshold() {
        let mut analyzer = ConnascenceAnalyzer::new();
        analyzer.set_module("test_module".to_string());

        // 3 args should not be flagged
        analyzer.record_position_dependency("small_fn", 3);
        assert_eq!(analyzer.stats.count(ConnascenceType::Position), 0);

        // 4+ args should be flagged
        analyzer.record_position_dependency("large_fn", 5);
        assert_eq!(analyzer.stats.count(ConnascenceType::Position), 1);
    }

    #[test]
    fn test_magic_number_detection() {
        let mut analyzer = ConnascenceAnalyzer::new();
        analyzer.set_module("test_module".to_string());

        // Acceptable values should not be flagged
        analyzer.record_magic_number("test", "0");
        analyzer.record_magic_number("test", "1");
        analyzer.record_magic_number("test", "true");
        assert_eq!(analyzer.stats.count(ConnascenceType::Meaning), 0);

        // Magic numbers should be flagged
        analyzer.record_magic_number("test", "42");
        analyzer.record_magic_number("test", "3.14159");
        assert_eq!(analyzer.stats.count(ConnascenceType::Meaning), 2);
    }

    #[test]
    fn test_algorithm_pattern_detection() {
        let code_with_encoding = "fn encode() {} fn decode() {}";
        let patterns = detect_algorithm_patterns(code_with_encoding);
        assert!(!patterns.is_empty());

        let code_without_patterns = "fn process() { let x = 1; }";
        let patterns = detect_algorithm_patterns(code_without_patterns);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_acceptable_literals() {
        assert!(is_acceptable_literal("0"));
        assert!(is_acceptable_literal("1"));
        assert!(is_acceptable_literal("true"));
        assert!(is_acceptable_literal("\"\""));
        assert!(!is_acceptable_literal("42"));
        assert!(!is_acceptable_literal("3.14159"));
    }
}
