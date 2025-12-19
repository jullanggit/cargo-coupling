//! A Philosophy of Software Design (APOSD) metrics detection
//!
//! Based on John Ousterhout's "A Philosophy of Software Design" (2nd Edition),
//! this module detects design patterns and anti-patterns that contribute to
//! software complexity.
//!
//! ## Key Concepts
//!
//! ### Deep vs Shallow Modules
//! - **Deep modules**: Simple interfaces hiding complex implementations (good)
//! - **Shallow modules**: Complex interfaces with simple implementations (bad)
//!
//! ### Information Hiding
//! - Modules should encapsulate design decisions
//! - Information leakage across boundaries indicates poor design
//!
//! ### Pass-through Methods
//! - Methods that only delegate to another method without adding value
//! - Indicates unclear responsibility division
//!
//! ## References
//! - John Ousterhout, "A Philosophy of Software Design" (2nd Edition, 2021)
//! - <https://web.stanford.edu/~ouster/cgi-bin/aposd.php>

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use syn::{visit::Visit, Expr, ItemFn, ItemImpl, Stmt};

use crate::config::AposdConfig;
use crate::metrics::ProjectMetrics;

/// Metrics for measuring module depth (interface vs implementation complexity)
#[derive(Debug, Clone, Default)]
pub struct ModuleDepthMetrics {
    /// Module name/path
    pub module_name: String,

    // Interface complexity metrics
    /// Number of public functions
    pub pub_function_count: usize,
    /// Number of public types (structs, enums, traits)
    pub pub_type_count: usize,
    /// Total parameters across all public functions
    pub total_pub_params: usize,
    /// Number of generic type parameters in public API
    pub generic_param_count: usize,
    /// Number of trait bounds in public API
    pub trait_bound_count: usize,
    /// Number of public constants
    pub pub_const_count: usize,

    // Implementation complexity metrics
    /// Total lines of code (excluding comments/blanks)
    pub implementation_loc: usize,
    /// Number of private functions
    pub private_function_count: usize,
    /// Number of private types
    pub private_type_count: usize,
    /// Cyclomatic complexity estimate (branches, loops, etc.)
    pub complexity_estimate: usize,
}

impl ModuleDepthMetrics {
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            ..Default::default()
        }
    }

    /// Calculate interface complexity score
    ///
    /// Higher score = more complex interface
    pub fn interface_complexity(&self) -> f64 {
        let fn_complexity = self.pub_function_count as f64 * 1.0;
        let type_complexity = self.pub_type_count as f64 * 0.5;
        let param_complexity = self.total_pub_params as f64 * 0.3;
        let generic_complexity = self.generic_param_count as f64 * 0.5;
        let trait_complexity = self.trait_bound_count as f64 * 0.3;
        let const_complexity = self.pub_const_count as f64 * 0.1;

        fn_complexity
            + type_complexity
            + param_complexity
            + generic_complexity
            + trait_complexity
            + const_complexity
    }

    /// Calculate implementation complexity score
    ///
    /// Higher score = more complex implementation (hidden behind interface)
    pub fn implementation_complexity(&self) -> f64 {
        let loc_complexity = self.implementation_loc as f64 * 0.1;
        let private_fn_complexity = self.private_function_count as f64 * 1.0;
        let private_type_complexity = self.private_type_count as f64 * 0.5;
        let cyclomatic_complexity = self.complexity_estimate as f64 * 0.5;

        loc_complexity + private_fn_complexity + private_type_complexity + cyclomatic_complexity
    }

    /// Calculate module depth ratio
    ///
    /// Depth = Implementation Complexity / Interface Complexity
    ///
    /// - High ratio (> 5.0): Deep module (good - hides complexity)
    /// - Low ratio (< 2.0): Shallow module (bad - interface as complex as implementation)
    ///
    /// Returns None if interface complexity is 0
    pub fn depth_ratio(&self) -> Option<f64> {
        let interface = self.interface_complexity();
        if interface < 0.01 {
            return None; // Avoid division by zero
        }

        let implementation = self.implementation_complexity();
        Some(implementation / interface)
    }

    /// Classify module depth
    pub fn depth_classification(&self) -> ModuleDepthClass {
        match self.depth_ratio() {
            None => ModuleDepthClass::Unknown,
            Some(ratio) if ratio >= 10.0 => ModuleDepthClass::VeryDeep,
            Some(ratio) if ratio >= 5.0 => ModuleDepthClass::Deep,
            Some(ratio) if ratio >= 2.0 => ModuleDepthClass::Moderate,
            Some(ratio) if ratio >= 1.0 => ModuleDepthClass::Shallow,
            Some(_) => ModuleDepthClass::VeryShallow,
        }
    }

    /// Check if this module is considered shallow (a red flag)
    pub fn is_shallow(&self) -> bool {
        matches!(
            self.depth_classification(),
            ModuleDepthClass::Shallow | ModuleDepthClass::VeryShallow
        )
    }

    /// Calculate average parameters per public function
    pub fn avg_params_per_function(&self) -> f64 {
        if self.pub_function_count == 0 {
            return 0.0;
        }
        self.total_pub_params as f64 / self.pub_function_count as f64
    }
}

/// Classification of module depth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleDepthClass {
    /// Ratio >= 10.0: Excellent abstraction (like Unix I/O)
    VeryDeep,
    /// Ratio >= 5.0: Good abstraction
    Deep,
    /// Ratio >= 2.0: Acceptable
    Moderate,
    /// Ratio >= 1.0: Interface nearly as complex as implementation
    Shallow,
    /// Ratio < 1.0: Interface MORE complex than implementation
    VeryShallow,
    /// Cannot calculate (no public interface)
    Unknown,
}

impl std::fmt::Display for ModuleDepthClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleDepthClass::VeryDeep => write!(f, "Very Deep"),
            ModuleDepthClass::Deep => write!(f, "Deep"),
            ModuleDepthClass::Moderate => write!(f, "Moderate"),
            ModuleDepthClass::Shallow => write!(f, "Shallow"),
            ModuleDepthClass::VeryShallow => write!(f, "Very Shallow"),
            ModuleDepthClass::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Metrics for detecting pass-through methods
#[derive(Debug, Clone)]
pub struct PassThroughMethodInfo {
    /// Method name
    pub method_name: String,
    /// Module where the method is defined
    pub module_name: String,
    /// The delegated method being called
    pub delegated_to: String,
    /// Number of parameters passed through unchanged
    pub params_passed_through: usize,
    /// Total number of parameters
    pub total_params: usize,
    /// Whether this is likely a pass-through (heuristic)
    pub is_passthrough: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

impl PassThroughMethodInfo {
    /// Calculate pass-through ratio
    pub fn passthrough_ratio(&self) -> f64 {
        if self.total_params == 0 {
            return 1.0; // No params = full pass-through
        }
        self.params_passed_through as f64 / self.total_params as f64
    }
}

/// Cognitive load metrics for a module
#[derive(Debug, Clone, Default)]
pub struct CognitiveLoadMetrics {
    /// Module name
    pub module_name: String,

    /// Number of public API items (functions, types, constants)
    pub public_api_count: usize,
    /// Number of dependencies (other modules this depends on)
    pub dependency_count: usize,
    /// Average function parameter count
    pub avg_param_count: f64,
    /// Number of different types used in public API
    pub type_variety: usize,
    /// Number of generic type parameters
    pub generics_count: usize,
    /// Number of trait bounds
    pub trait_bounds_count: usize,
    /// Maximum nesting depth in the module
    pub max_nesting_depth: usize,
    /// Number of control flow branches (if, match, loop)
    pub branch_count: usize,
}

impl CognitiveLoadMetrics {
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            ..Default::default()
        }
    }

    /// Calculate cognitive load score
    ///
    /// Higher score = higher cognitive load (harder to understand)
    pub fn cognitive_load_score(&self) -> f64 {
        // Weights based on cognitive psychology research
        let api_weight = self.public_api_count as f64 * 0.25;
        let dep_weight = self.dependency_count as f64 * 0.20;
        let param_weight = self.avg_param_count * 0.15;
        let type_weight = self.type_variety as f64 * 0.10;
        let generic_weight = self.generics_count as f64 * 0.10;
        let trait_weight = self.trait_bounds_count as f64 * 0.10;
        let nesting_weight = self.max_nesting_depth as f64 * 0.05;
        let branch_weight = self.branch_count as f64 * 0.05;

        api_weight
            + dep_weight
            + param_weight
            + type_weight
            + generic_weight
            + trait_weight
            + nesting_weight
            + branch_weight
    }

    /// Classify cognitive load level
    pub fn load_classification(&self) -> CognitiveLoadLevel {
        let score = self.cognitive_load_score();
        match score {
            s if s < 5.0 => CognitiveLoadLevel::Low,
            s if s < 15.0 => CognitiveLoadLevel::Moderate,
            s if s < 30.0 => CognitiveLoadLevel::High,
            _ => CognitiveLoadLevel::VeryHigh,
        }
    }

    /// Check if cognitive load is problematic
    pub fn is_high_load(&self) -> bool {
        matches!(
            self.load_classification(),
            CognitiveLoadLevel::High | CognitiveLoadLevel::VeryHigh
        )
    }
}

/// Classification of cognitive load
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveLoadLevel {
    /// Easy to understand
    Low,
    /// Manageable complexity
    Moderate,
    /// Requires significant effort to understand
    High,
    /// Overwhelming complexity
    VeryHigh,
}

impl std::fmt::Display for CognitiveLoadLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CognitiveLoadLevel::Low => write!(f, "Low"),
            CognitiveLoadLevel::Moderate => write!(f, "Moderate"),
            CognitiveLoadLevel::High => write!(f, "High"),
            CognitiveLoadLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

/// Summary of APOSD metrics for a project
#[derive(Debug, Default)]
pub struct AposdAnalysis {
    /// Module depth metrics for each module
    pub module_depths: HashMap<String, ModuleDepthMetrics>,
    /// Detected pass-through methods
    pub passthrough_methods: Vec<PassThroughMethodInfo>,
    /// Cognitive load metrics for each module
    pub cognitive_loads: HashMap<String, CognitiveLoadMetrics>,
}

impl AposdAnalysis {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all shallow modules
    pub fn shallow_modules(&self) -> Vec<&ModuleDepthMetrics> {
        self.module_depths
            .values()
            .filter(|m| m.is_shallow())
            .collect()
    }

    /// Get all high cognitive load modules
    pub fn high_load_modules(&self) -> Vec<&CognitiveLoadMetrics> {
        self.cognitive_loads
            .values()
            .filter(|m| m.is_high_load())
            .collect()
    }

    /// Get pass-through methods with high confidence
    pub fn confirmed_passthroughs(&self) -> Vec<&PassThroughMethodInfo> {
        self.passthrough_methods
            .iter()
            .filter(|m| m.is_passthrough && m.confidence > 0.7)
            .collect()
    }

    /// Calculate overall project depth score
    pub fn average_depth_ratio(&self) -> Option<f64> {
        let ratios: Vec<f64> = self
            .module_depths
            .values()
            .filter_map(|m| m.depth_ratio())
            .collect();

        if ratios.is_empty() {
            return None;
        }

        Some(ratios.iter().sum::<f64>() / ratios.len() as f64)
    }

    /// Calculate overall cognitive load score
    pub fn average_cognitive_load(&self) -> f64 {
        if self.cognitive_loads.is_empty() {
            return 0.0;
        }

        let sum: f64 = self
            .cognitive_loads
            .values()
            .map(|m| m.cognitive_load_score())
            .sum();

        sum / self.cognitive_loads.len() as f64
    }

    /// Count of issues by category
    pub fn issue_counts(&self) -> AposdIssueCounts {
        AposdIssueCounts {
            shallow_modules: self.shallow_modules().len(),
            passthrough_methods: self.confirmed_passthroughs().len(),
            high_cognitive_load: self.high_load_modules().len(),
        }
    }
}

/// Summary counts of APOSD issues
#[derive(Debug, Clone, Default)]
pub struct AposdIssueCounts {
    pub shallow_modules: usize,
    pub passthrough_methods: usize,
    pub high_cognitive_load: usize,
}

impl AposdIssueCounts {
    /// Total number of APOSD issues
    pub fn total(&self) -> usize {
        self.shallow_modules + self.passthrough_methods + self.high_cognitive_load
    }

    /// Check if there are any issues
    pub fn has_issues(&self) -> bool {
        self.total() > 0
    }
}

// ============================================================================
// APOSD Analyzer - Analyzes project for APOSD patterns
// ============================================================================

/// Analyze a project for APOSD metrics
///
/// This analyzer computes module depth, cognitive load, and detects
/// pass-through methods based on AST analysis.
pub fn analyze_aposd(
    _path: &Path,
    project_metrics: &ProjectMetrics,
    config: &AposdConfig,
) -> AposdAnalysis {
    let mut analysis = AposdAnalysis::new();

    // Analyze each module for depth and cognitive load
    for (module_name, module_metrics) in &project_metrics.modules {
        // Calculate module depth
        let mut depth = ModuleDepthMetrics::new(module_name.clone());

        // Count public items from the module metrics
        depth.pub_type_count = module_metrics.public_type_count();
        depth.private_type_count = module_metrics.private_type_count();

        // Analyze the source file for more detailed metrics
        if let Ok(content) = fs::read_to_string(&module_metrics.path) {
            let file_metrics = analyze_file_for_aposd(&content, config);
            depth.pub_function_count = file_metrics.pub_function_count;
            depth.total_pub_params = file_metrics.total_pub_params;
            depth.generic_param_count = file_metrics.generic_param_count;
            depth.implementation_loc = file_metrics.implementation_loc;
            depth.private_function_count = file_metrics.private_function_count;
            depth.complexity_estimate = file_metrics.complexity_estimate;

            // Detect pass-through methods
            for pt in file_metrics.passthrough_candidates {
                analysis.passthrough_methods.push(PassThroughMethodInfo {
                    method_name: pt.method_name,
                    module_name: module_name.clone(),
                    delegated_to: pt.delegated_to,
                    params_passed_through: pt.params_passed_through,
                    total_params: pt.total_params,
                    is_passthrough: pt.is_passthrough,
                    confidence: pt.confidence,
                });
            }
        }

        analysis
            .module_depths
            .insert(module_name.clone(), depth.clone());

        // Calculate cognitive load
        let mut cognitive = CognitiveLoadMetrics::new(module_name.clone());
        cognitive.public_api_count =
            depth.pub_function_count + depth.pub_type_count + depth.pub_const_count;
        cognitive.dependency_count =
            module_metrics.external_deps.len() + module_metrics.internal_deps.len();
        cognitive.avg_param_count = depth.avg_params_per_function();
        cognitive.generics_count = depth.generic_param_count;
        cognitive.trait_bounds_count = depth.trait_bound_count;
        cognitive.branch_count = depth.complexity_estimate;

        analysis
            .cognitive_loads
            .insert(module_name.clone(), cognitive);
    }

    analysis
}

/// Internal file metrics from AST analysis
struct FileAposdMetrics {
    pub_function_count: usize,
    total_pub_params: usize,
    generic_param_count: usize,
    implementation_loc: usize,
    private_function_count: usize,
    complexity_estimate: usize,
    passthrough_candidates: Vec<PassThroughCandidate>,
}

struct PassThroughCandidate {
    method_name: String,
    delegated_to: String,
    params_passed_through: usize,
    total_params: usize,
    is_passthrough: bool,
    confidence: f64,
}

/// AST visitor for APOSD metrics
struct AposdVisitor<'a> {
    pub_function_count: usize,
    private_function_count: usize,
    total_pub_params: usize,
    generic_param_count: usize,
    complexity_estimate: usize,
    line_count: usize,
    passthrough_candidates: Vec<PassThroughCandidate>,
    config: &'a AposdConfig,
}

impl<'a> AposdVisitor<'a> {
    fn new(config: &'a AposdConfig) -> Self {
        Self {
            pub_function_count: 0,
            private_function_count: 0,
            total_pub_params: 0,
            generic_param_count: 0,
            complexity_estimate: 0,
            line_count: 0,
            passthrough_candidates: Vec::new(),
            config,
        }
    }

    fn is_public(&self, vis: &syn::Visibility) -> bool {
        matches!(vis, syn::Visibility::Public(_))
    }

    fn count_params(&self, sig: &syn::Signature) -> usize {
        sig.inputs
            .iter()
            .filter(|arg| !matches!(arg, syn::FnArg::Receiver(_)))
            .count()
    }

    fn count_generics(&self, generics: &syn::Generics) -> usize {
        generics.type_params().count() + generics.lifetimes().count()
    }

    /// Check if a method name is a Rust idiomatic pattern that should not be flagged
    fn is_rust_idiomatic_method(&self, name: &str) -> bool {
        // Check if Rust idiom exclusion is disabled
        if !self.config.exclude_rust_idioms {
            // Only check custom exclusions
            return self.is_custom_excluded_method(name);
        }

        // Rust-specific patterns that are intentionally simple delegations:

        // 1. Conversion methods (AsRef, AsMut, Into, From patterns)
        if name.starts_with("as_")
            || name.starts_with("into_")
            || name.starts_with("from_")
            || name.starts_with("to_")
        {
            return true;
        }

        // 2. Accessor patterns (getters/setters)
        if name.starts_with("get_")
            || name.starts_with("set_")
            || name.ends_with("_ref")
            || name.ends_with("_mut")
        {
            return true;
        }

        // 3. Common trait method names
        let trait_methods = [
            "deref",
            "deref_mut",
            "as_ref",
            "as_mut",
            "borrow",
            "borrow_mut",
            "clone",
            "default",
            "eq",
            "ne",
            "partial_cmp",
            "cmp",
            "hash",
            "fmt",
            "drop",
            "index",
            "index_mut",
        ];
        if trait_methods.contains(&name) {
            return true;
        }

        // 4. Builder pattern methods (typically return Self)
        if name.starts_with("with_") || name.starts_with("and_") {
            return true;
        }

        // 5. Iterator adaptor patterns
        if name == "iter" || name == "iter_mut" || name == "into_iter" {
            return true;
        }

        // 6. Common simple accessors
        let simple_accessors = ["len", "is_empty", "capacity", "inner", "get", "new"];
        if simple_accessors.contains(&name) {
            return true;
        }

        // 7. Check custom exclusions from config
        self.is_custom_excluded_method(name)
    }

    /// Check if a method is excluded via custom config
    fn is_custom_excluded_method(&self, name: &str) -> bool {
        // Check custom exclude_prefixes
        for prefix in &self.config.exclude_prefixes {
            if name.starts_with(prefix) {
                return true;
            }
        }

        // Check custom exclude_methods
        if self.config.exclude_methods.contains(&name.to_string()) {
            return true;
        }

        false
    }

    /// Check if the expression uses error propagation with `?` operator
    fn uses_error_propagation(expr: &Expr) -> bool {
        matches!(expr, Expr::Try(_))
    }

    /// Check if a function body is a simple delegation (pass-through)
    fn check_passthrough(&mut self, name: &str, sig: &syn::Signature, block: &syn::Block) {
        // A pass-through method typically has:
        // 1. A single statement or expression
        // 2. That expression is a method call or function call
        // 3. Most parameters are passed through unchanged

        // Skip Rust idiomatic patterns - these are intentional, not design issues
        if self.is_rust_idiomatic_method(name) {
            return;
        }

        let total_params = self.count_params(sig);

        // Check if block has a single expression or single return
        if block.stmts.len() != 1 {
            return;
        }

        let is_single_expr = match &block.stmts[0] {
            Stmt::Expr(expr, _) => self.is_simple_delegation(expr),
            _ => false,
        };

        if !is_single_expr {
            return;
        }

        // Analyze the single statement
        if let Some(Stmt::Expr(expr, _)) = block.stmts.first() {
            // Skip methods that use `?` for error propagation - this is idiomatic Rust
            if Self::uses_error_propagation(expr) {
                return;
            }

            if let Some((delegated_to, passed_through)) = self.analyze_delegation(expr) {
                let passthrough_ratio = if total_params > 0 {
                    passed_through as f64 / total_params as f64
                } else {
                    1.0
                };

                // Consider it a pass-through if most params are passed through
                let is_passthrough = passthrough_ratio >= 0.8 && total_params > 0;
                let confidence = passthrough_ratio;

                self.passthrough_candidates.push(PassThroughCandidate {
                    method_name: name.to_string(),
                    delegated_to,
                    params_passed_through: passed_through,
                    total_params,
                    is_passthrough,
                    confidence,
                });
            }
        }
    }

    fn is_simple_delegation(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::MethodCall(_) | Expr::Call(_) | Expr::Try(_) | Expr::Await(_)
        )
    }

    fn analyze_delegation(&self, expr: &Expr) -> Option<(String, usize)> {
        match expr {
            Expr::MethodCall(mc) => {
                let method_name = mc.method.to_string();
                let args_count = mc.args.len();
                Some((format!("self.{}", method_name), args_count))
            }
            Expr::Call(call) => {
                // Try to extract a readable name from the function expression
                let callee = match call.func.as_ref() {
                    Expr::Path(path) => {
                        path.path
                            .segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::")
                    }
                    Expr::Field(field) => {
                        match &field.member {
                            syn::Member::Named(ident) => format!("_.{}", ident),
                            syn::Member::Unnamed(index) => format!("_.{}", index.index),
                        }
                    }
                    _ => "unknown".to_string(),
                };
                let args_count = call.args.len();
                Some((callee, args_count))
            }
            Expr::Try(try_expr) => self.analyze_delegation(&try_expr.expr),
            Expr::Await(await_expr) => self.analyze_delegation(&await_expr.base),
            _ => None,
        }
    }
}

impl<'ast, 'a> Visit<'ast> for AposdVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if self.is_public(&node.vis) {
            self.pub_function_count += 1;
            self.total_pub_params += self.count_params(&node.sig);
            self.generic_param_count += self.count_generics(&node.sig.generics);
        } else {
            self.private_function_count += 1;
        }

        // Check for pass-through pattern
        self.check_passthrough(&node.sig.ident.to_string(), &node.sig, &node.block);

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            if let syn::ImplItem::Fn(method) = item {
                let is_pub = matches!(method.vis, syn::Visibility::Public(_));

                if is_pub {
                    self.pub_function_count += 1;
                    self.total_pub_params += self.count_params(&method.sig);
                    self.generic_param_count += self.count_generics(&method.sig.generics);
                } else {
                    self.private_function_count += 1;
                }

                // Check for pass-through pattern
                self.check_passthrough(
                    &method.sig.ident.to_string(),
                    &method.sig,
                    &method.block,
                );
            }
        }

        syn::visit::visit_item_impl(self, node);
    }

    // Count complexity indicators
    fn visit_expr(&mut self, node: &'ast Expr) {
        match node {
            Expr::If(_) | Expr::Match(_) | Expr::While(_) | Expr::ForLoop(_) | Expr::Loop(_) => {
                self.complexity_estimate += 1;
            }
            _ => {}
        }
        syn::visit::visit_expr(self, node);
    }
}

/// Analyze a file for APOSD metrics
fn analyze_file_for_aposd(content: &str, config: &AposdConfig) -> FileAposdMetrics {
    let mut visitor = AposdVisitor::new(config);
    visitor.line_count = content.lines().count();

    if let Ok(syntax) = syn::parse_file(content) {
        visitor.visit_file(&syntax);
    }

    FileAposdMetrics {
        pub_function_count: visitor.pub_function_count,
        total_pub_params: visitor.total_pub_params,
        generic_param_count: visitor.generic_param_count,
        implementation_loc: visitor.line_count,
        private_function_count: visitor.private_function_count,
        complexity_estimate: visitor.complexity_estimate,
        passthrough_candidates: visitor.passthrough_candidates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_depth_calculation() {
        let mut metrics = ModuleDepthMetrics::new("test_module".to_string());

        // Simple interface, complex implementation = deep module
        metrics.pub_function_count = 2;
        metrics.total_pub_params = 4;
        metrics.implementation_loc = 200;
        metrics.private_function_count = 10;
        metrics.complexity_estimate = 20;

        let ratio = metrics.depth_ratio().unwrap();
        assert!(ratio > 5.0, "Expected deep module, got ratio: {}", ratio);
        // VeryDeep because ratio is >= 10.0
        assert!(
            matches!(
                metrics.depth_classification(),
                ModuleDepthClass::Deep | ModuleDepthClass::VeryDeep
            ),
            "Expected Deep or VeryDeep, got {:?}",
            metrics.depth_classification()
        );
    }

    #[test]
    fn test_shallow_module_detection() {
        let mut metrics = ModuleDepthMetrics::new("shallow_module".to_string());

        // Complex interface, simple implementation = shallow module
        metrics.pub_function_count = 10;
        metrics.total_pub_params = 30;
        metrics.pub_type_count = 5;
        metrics.implementation_loc = 50;
        metrics.private_function_count = 2;

        assert!(metrics.is_shallow(), "Expected shallow module");
    }

    #[test]
    fn test_cognitive_load_scoring() {
        let mut load = CognitiveLoadMetrics::new("test".to_string());
        load.public_api_count = 5;
        load.dependency_count = 3;
        load.avg_param_count = 2.0;

        let score = load.cognitive_load_score();
        assert!(score > 0.0);
        assert_eq!(load.load_classification(), CognitiveLoadLevel::Low);

        // High load module
        let mut high_load = CognitiveLoadMetrics::new("complex".to_string());
        high_load.public_api_count = 50;
        high_load.dependency_count = 20;
        high_load.avg_param_count = 5.0;
        high_load.generics_count = 10;
        high_load.trait_bounds_count = 15;

        assert!(high_load.is_high_load());
    }

    #[test]
    fn test_passthrough_ratio() {
        let passthrough = PassThroughMethodInfo {
            method_name: "delegate".to_string(),
            module_name: "wrapper".to_string(),
            delegated_to: "inner.method".to_string(),
            params_passed_through: 3,
            total_params: 3,
            is_passthrough: true,
            confidence: 0.9,
        };

        assert_eq!(passthrough.passthrough_ratio(), 1.0);
    }

    #[test]
    fn test_aposd_analysis_summary() {
        let mut analysis = AposdAnalysis::new();

        // Add a shallow module
        let mut shallow = ModuleDepthMetrics::new("shallow".to_string());
        shallow.pub_function_count = 10;
        shallow.total_pub_params = 20;
        shallow.implementation_loc = 30;
        analysis
            .module_depths
            .insert("shallow".to_string(), shallow);

        // Add a deep module
        let mut deep = ModuleDepthMetrics::new("deep".to_string());
        deep.pub_function_count = 2;
        deep.implementation_loc = 500;
        deep.private_function_count = 20;
        analysis.module_depths.insert("deep".to_string(), deep);

        let counts = analysis.issue_counts();
        assert_eq!(counts.shallow_modules, 1);
    }
}
