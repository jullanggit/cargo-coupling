//! Temporal Coupling detection through static analysis
//!
//! Temporal coupling occurs when components must be used in a specific order.
//! This module detects potential temporal coupling through heuristic patterns:
//!
//! 1. **Paired Operations**: open/close, lock/unlock, begin/commit
//! 2. **Lifecycle Methods**: init, setup, start, stop, cleanup
//! 3. **State Dependencies**: Methods that check initialization state
//! 4. **Rust-specific Patterns**: Drop trait, MutexGuard, async spawn/join
//!
//! Note: This is heuristic-based detection. Runtime order cannot be
//! fully determined through static analysis.

use std::collections::HashMap;

/// Types of temporal coupling patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemporalPattern {
    /// Paired operations that must be balanced (open/close, lock/unlock)
    PairedOperation {
        open_method: String,
        close_method: String,
    },
    /// Lifecycle methods that suggest initialization order
    LifecycleSequence {
        phase: LifecyclePhase,
        method_name: String,
    },
    /// State check suggesting temporal dependency
    StateCheck {
        check_method: String,
        implied_prerequisite: String,
    },
    /// Rust-specific: Drop impl provides cleanup
    RustDropImpl { type_name: String },
    /// Rust-specific: Guard pattern (MutexGuard, RwLockGuard, etc.)
    RustGuardPattern {
        guard_type: String,
        resource: String,
    },
    /// Rust-specific: Async spawn without join
    RustAsyncSpawnWithoutJoin,
    /// Rust-specific: Unsafe block with manual resource management
    RustUnsafeManualResource { operation: String },
    /// Rust-specific: Builder pattern detected
    RustBuilderPattern {
        type_name: String,
        required_methods: Vec<String>,
    },
}

/// Lifecycle phases for temporal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LifecyclePhase {
    /// Phase 1: Construction/Creation
    Create = 0,
    /// Phase 2: Configuration
    Configure = 1,
    /// Phase 3: Initialization
    Initialize = 2,
    /// Phase 4: Start/Connect
    Start = 3,
    /// Phase 5: Running/Active
    Active = 4,
    /// Phase 6: Stop/Disconnect
    Stop = 5,
    /// Phase 7: Cleanup/Destroy
    Cleanup = 6,
}

impl LifecyclePhase {
    pub fn description(&self) -> &'static str {
        match self {
            LifecyclePhase::Create => "Object creation",
            LifecyclePhase::Configure => "Configuration",
            LifecyclePhase::Initialize => "Initialization",
            LifecyclePhase::Start => "Start/Connect",
            LifecyclePhase::Active => "Active operation",
            LifecyclePhase::Stop => "Stop/Disconnect",
            LifecyclePhase::Cleanup => "Cleanup/Destroy",
        }
    }
}

/// A detected temporal coupling instance
#[derive(Debug, Clone)]
pub struct TemporalCouplingInstance {
    /// The pattern type
    pub pattern: TemporalPattern,
    /// Source module/file
    pub source: String,
    /// Severity (0.0 - 1.0)
    pub severity: f64,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggestion: String,
}

/// Paired operation definition
#[derive(Debug, Clone)]
struct PairedOp {
    open: &'static str,
    close: &'static str,
    severity: f64,
}

/// Statistics about temporal coupling
#[derive(Debug, Clone, Default)]
pub struct TemporalCouplingStats {
    /// Paired operations found
    pub paired_operations: HashMap<String, PairedOperationStats>,
    /// Lifecycle methods by phase
    pub lifecycle_methods: HashMap<LifecyclePhase, Vec<String>>,
    /// State check patterns
    pub state_checks: Vec<String>,
    /// Total issues detected
    pub total_issues: usize,
    /// Rust-specific: Types with Drop impl
    pub drop_impls: Vec<String>,
    /// Rust-specific: Guard patterns used
    pub guard_patterns: Vec<String>,
    /// Rust-specific: Async spawn count
    pub async_spawns: usize,
    /// Rust-specific: Async join count
    pub async_joins: usize,
    /// Rust-specific: Unsafe blocks with allocation
    pub unsafe_allocations: Vec<String>,
    /// Rust-specific: Builder patterns detected
    pub builder_patterns: Vec<String>,
}

/// Stats for a specific paired operation type
#[derive(Debug, Clone, Default)]
pub struct PairedOperationStats {
    pub open_count: usize,
    pub close_count: usize,
    pub locations: Vec<String>,
}

/// Temporal coupling analyzer
#[derive(Debug, Default)]
pub struct TemporalAnalyzer {
    /// Detected instances
    pub instances: Vec<TemporalCouplingInstance>,
    /// Statistics
    pub stats: TemporalCouplingStats,
    /// Current module being analyzed
    current_module: String,
    /// Method calls found (for paired operation tracking)
    method_calls: HashMap<String, Vec<String>>,
    /// Function definitions found
    function_defs: Vec<(String, String)>, // (module, function_name)
    /// Rust-specific: Drop impls found
    drop_impls: Vec<(String, String)>, // (module, type_name)
    /// Rust-specific: Guard usages
    guard_usages: Vec<(String, String)>, // (module, guard_type)
    /// Rust-specific: Async spawns
    async_spawns: Vec<String>,
    /// Rust-specific: Async joins
    async_joins: Vec<String>,
    /// Rust-specific: Unsafe allocations
    unsafe_allocs: Vec<(String, String)>, // (module, operation)
    /// Rust-specific: Builder pattern types
    builder_types: Vec<(String, Vec<String>)>, // (type_name, builder_methods)
}

impl TemporalAnalyzer {
    /// Known paired operations
    const PAIRED_OPS: &'static [PairedOp] = &[
        PairedOp {
            open: "open",
            close: "close",
            severity: 0.8,
        },
        PairedOp {
            open: "lock",
            close: "unlock",
            severity: 0.9,
        },
        PairedOp {
            open: "acquire",
            close: "release",
            severity: 0.9,
        },
        PairedOp {
            open: "begin",
            close: "commit",
            severity: 0.7,
        },
        PairedOp {
            open: "begin",
            close: "end",
            severity: 0.6,
        },
        PairedOp {
            open: "start",
            close: "stop",
            severity: 0.7,
        },
        PairedOp {
            open: "connect",
            close: "disconnect",
            severity: 0.8,
        },
        PairedOp {
            open: "enter",
            close: "exit",
            severity: 0.7,
        },
        PairedOp {
            open: "push",
            close: "pop",
            severity: 0.5,
        },
        PairedOp {
            open: "subscribe",
            close: "unsubscribe",
            severity: 0.6,
        },
        PairedOp {
            open: "register",
            close: "unregister",
            severity: 0.6,
        },
        PairedOp {
            open: "enable",
            close: "disable",
            severity: 0.5,
        },
        PairedOp {
            open: "activate",
            close: "deactivate",
            severity: 0.6,
        },
        PairedOp {
            open: "attach",
            close: "detach",
            severity: 0.6,
        },
        PairedOp {
            open: "bind",
            close: "unbind",
            severity: 0.7,
        },
        PairedOp {
            open: "mount",
            close: "unmount",
            severity: 0.8,
        },
        PairedOp {
            open: "init",
            close: "deinit",
            severity: 0.7,
        },
        PairedOp {
            open: "setup",
            close: "teardown",
            severity: 0.7,
        },
        PairedOp {
            open: "create",
            close: "destroy",
            severity: 0.7,
        },
        PairedOp {
            open: "alloc",
            close: "free",
            severity: 0.9,
        },
        PairedOp {
            open: "malloc",
            close: "free",
            severity: 0.9,
        },
        PairedOp {
            open: "borrow",
            close: "return",
            severity: 0.6,
        },
        PairedOp {
            open: "checkout",
            close: "checkin",
            severity: 0.6,
        },
    ];

    /// Lifecycle method patterns
    const LIFECYCLE_PATTERNS: &'static [(LifecyclePhase, &'static [&'static str])] = &[
        (
            LifecyclePhase::Create,
            &["new", "create", "build", "construct", "make"],
        ),
        (
            LifecyclePhase::Configure,
            &[
                "configure",
                "config",
                "set_config",
                "with_config",
                "options",
            ],
        ),
        (
            LifecyclePhase::Initialize,
            &["init", "initialize", "setup", "prepare", "bootstrap"],
        ),
        (
            LifecyclePhase::Start,
            &[
                "start", "begin", "run", "launch", "open", "connect", "activate",
            ],
        ),
        (
            LifecyclePhase::Active,
            &["process", "execute", "handle", "perform", "do_work"],
        ),
        (
            LifecyclePhase::Stop,
            &[
                "stop",
                "end",
                "halt",
                "pause",
                "close",
                "disconnect",
                "deactivate",
            ],
        ),
        (
            LifecyclePhase::Cleanup,
            &[
                "cleanup", "clean", "dispose", "destroy", "drop", "finalize", "shutdown",
                "teardown",
            ],
        ),
    ];

    /// State check patterns that imply temporal dependency
    const STATE_CHECK_PATTERNS: &'static [(&'static str, &'static str)] = &[
        ("is_initialized", "init/initialize"),
        ("is_connected", "connect"),
        ("is_open", "open"),
        ("is_started", "start"),
        ("is_ready", "init/prepare"),
        ("is_running", "start/run"),
        ("is_active", "activate/start"),
        ("is_configured", "configure"),
        ("is_setup", "setup"),
        ("has_started", "start"),
        ("was_initialized", "init"),
        ("check_initialized", "init"),
        ("ensure_initialized", "init"),
        ("assert_initialized", "init"),
        ("require_connection", "connect"),
    ];

    /// Rust-specific: Guard types that auto-release
    const RUST_GUARD_TYPES: &'static [&'static str] = &[
        "MutexGuard",
        "RwLockReadGuard",
        "RwLockWriteGuard",
        "RefCell",
        "Ref",
        "RefMut",
        "ScopedJoinHandle",
        "Guard",
        "ScopeGuard",
        "Entered", // tracing span guard
    ];

    /// Rust-specific: Unsafe allocation patterns
    const RUST_UNSAFE_ALLOC_PATTERNS: &'static [&'static str] = &[
        "alloc",
        "dealloc",
        "realloc",
        "Box::from_raw",
        "Box::into_raw",
        "Vec::from_raw_parts",
        "String::from_raw_parts",
        "ptr::read",
        "ptr::write",
        "ManuallyDrop",
        "mem::forget",
        "mem::transmute",
    ];

    /// Rust-specific: Async spawn patterns
    const RUST_ASYNC_SPAWN_PATTERNS: &'static [&'static str] = &[
        "spawn",
        "spawn_blocking",
        "spawn_local",
        "task::spawn",
        "tokio::spawn",
        "async_std::spawn",
        "rayon::spawn",
    ];

    /// Rust-specific: Async join patterns
    const RUST_ASYNC_JOIN_PATTERNS: &'static [&'static str] =
        &["join", "join_all", "await", "block_on", "JoinHandle"];

    pub fn new() -> Self {
        Self::default()
    }

    /// Set current module context
    pub fn set_module(&mut self, module: String) {
        self.current_module = module;
    }

    /// Record a method/function call
    pub fn record_call(&mut self, method_name: &str) {
        let name = method_name.to_lowercase();
        self.method_calls
            .entry(name)
            .or_default()
            .push(self.current_module.clone());
    }

    /// Record a function definition
    pub fn record_function_def(&mut self, function_name: &str) {
        self.function_defs
            .push((self.current_module.clone(), function_name.to_lowercase()));
    }

    /// Record a Drop impl
    pub fn record_drop_impl(&mut self, type_name: &str) {
        self.drop_impls
            .push((self.current_module.clone(), type_name.to_string()));
    }

    /// Record a guard usage
    pub fn record_guard_usage(&mut self, guard_type: &str) {
        self.guard_usages
            .push((self.current_module.clone(), guard_type.to_string()));
    }

    /// Record an async spawn
    pub fn record_async_spawn(&mut self, spawn_type: &str) {
        self.async_spawns
            .push(format!("{}::{}", self.current_module, spawn_type));
    }

    /// Record an async join
    pub fn record_async_join(&mut self, join_type: &str) {
        self.async_joins
            .push(format!("{}::{}", self.current_module, join_type));
    }

    /// Record unsafe allocation
    pub fn record_unsafe_alloc(&mut self, operation: &str) {
        self.unsafe_allocs
            .push((self.current_module.clone(), operation.to_string()));
    }

    /// Record builder pattern
    pub fn record_builder_pattern(&mut self, type_name: &str, methods: Vec<String>) {
        self.builder_types.push((type_name.to_string(), methods));
    }

    /// Analyze collected data for temporal coupling patterns
    pub fn analyze(&mut self) {
        self.detect_paired_operation_imbalance();
        self.detect_lifecycle_patterns();
        self.detect_state_checks();
        self.detect_rust_patterns();
    }

    /// Detect imbalanced paired operations
    fn detect_paired_operation_imbalance(&mut self) {
        for paired in Self::PAIRED_OPS {
            let open_calls = self
                .method_calls
                .get(paired.open)
                .map(|v| v.len())
                .unwrap_or(0);
            let close_calls = self
                .method_calls
                .get(paired.close)
                .map(|v| v.len())
                .unwrap_or(0);

            if open_calls > 0 || close_calls > 0 {
                let stats = self
                    .stats
                    .paired_operations
                    .entry(format!("{}/{}", paired.open, paired.close))
                    .or_default();
                stats.open_count = open_calls;
                stats.close_count = close_calls;

                // Record locations
                if let Some(locs) = self.method_calls.get(paired.open) {
                    stats.locations.extend(locs.iter().cloned());
                }
                if let Some(locs) = self.method_calls.get(paired.close) {
                    stats.locations.extend(locs.iter().cloned());
                }

                // Detect imbalance
                if open_calls != close_calls && open_calls > 0 && close_calls > 0 {
                    let (description, suggestion) = if open_calls > close_calls {
                        (
                            format!(
                                "More {}() calls ({}) than {}() calls ({})",
                                paired.open, open_calls, paired.close, close_calls
                            ),
                            format!(
                                "Ensure every {}() has a matching {}(). Consider using RAII pattern or Drop trait.",
                                paired.open, paired.close
                            ),
                        )
                    } else {
                        (
                            format!(
                                "More {}() calls ({}) than {}() calls ({})",
                                paired.close, close_calls, paired.open, open_calls
                            ),
                            format!(
                                "Check if {}() is called without prior {}()",
                                paired.close, paired.open
                            ),
                        )
                    };

                    self.instances.push(TemporalCouplingInstance {
                        pattern: TemporalPattern::PairedOperation {
                            open_method: paired.open.to_string(),
                            close_method: paired.close.to_string(),
                        },
                        source: "project-wide".to_string(),
                        severity: paired.severity,
                        description,
                        suggestion,
                    });
                    self.stats.total_issues += 1;
                }
            }
        }
    }

    /// Detect lifecycle method patterns
    fn detect_lifecycle_patterns(&mut self) {
        let mut found_phases: Vec<(LifecyclePhase, String, String)> = Vec::new();

        for (module, func_name) in &self.function_defs {
            for (phase, patterns) in Self::LIFECYCLE_PATTERNS {
                for pattern in *patterns {
                    if func_name.contains(pattern) {
                        found_phases.push((*phase, module.clone(), func_name.clone()));
                        self.stats
                            .lifecycle_methods
                            .entry(*phase)
                            .or_default()
                            .push(format!("{}::{}", module, func_name));
                        break;
                    }
                }
            }
        }

        // Check for missing lifecycle phases (heuristic)
        let has_init = self
            .stats
            .lifecycle_methods
            .contains_key(&LifecyclePhase::Initialize);
        let has_cleanup = self
            .stats
            .lifecycle_methods
            .contains_key(&LifecyclePhase::Cleanup);
        let has_start = self
            .stats
            .lifecycle_methods
            .contains_key(&LifecyclePhase::Start);
        let has_stop = self
            .stats
            .lifecycle_methods
            .contains_key(&LifecyclePhase::Stop);

        // Warn if init exists but no cleanup
        if has_init && !has_cleanup {
            self.instances.push(TemporalCouplingInstance {
                pattern: TemporalPattern::LifecycleSequence {
                    phase: LifecyclePhase::Initialize,
                    method_name: "init*".to_string(),
                },
                source: "project-wide".to_string(),
                severity: 0.5,
                description: "Initialization methods found but no cleanup/teardown methods"
                    .to_string(),
                suggestion: "Consider adding cleanup methods to properly release resources"
                    .to_string(),
            });
            self.stats.total_issues += 1;
        }

        // Warn if start exists but no stop
        if has_start && !has_stop {
            self.instances.push(TemporalCouplingInstance {
                pattern: TemporalPattern::LifecycleSequence {
                    phase: LifecyclePhase::Start,
                    method_name: "start*".to_string(),
                },
                source: "project-wide".to_string(),
                severity: 0.5,
                description: "Start methods found but no stop methods".to_string(),
                suggestion: "Consider adding stop/shutdown methods for graceful termination"
                    .to_string(),
            });
            self.stats.total_issues += 1;
        }
    }

    /// Detect state check patterns
    fn detect_state_checks(&mut self) {
        for (module, func_name) in &self.function_defs {
            for (check_pattern, prerequisite) in Self::STATE_CHECK_PATTERNS {
                if func_name.contains(check_pattern) {
                    self.stats
                        .state_checks
                        .push(format!("{}::{}", module, func_name));

                    self.instances.push(TemporalCouplingInstance {
                        pattern: TemporalPattern::StateCheck {
                            check_method: func_name.clone(),
                            implied_prerequisite: prerequisite.to_string(),
                        },
                        source: module.clone(),
                        severity: 0.4,
                        description: format!(
                            "State check '{}' implies temporal dependency on {}",
                            func_name, prerequisite
                        ),
                        suggestion: "Document the required call order or use type-state pattern to enforce it at compile time".to_string(),
                    });
                    self.stats.total_issues += 1;
                    break;
                }
            }
        }
    }

    /// Detect Rust-specific temporal patterns
    fn detect_rust_patterns(&mut self) {
        // Record Drop impls (positive pattern - indicates RAII)
        for (module, type_name) in &self.drop_impls {
            self.stats
                .drop_impls
                .push(format!("{}::{}", module, type_name));
        }

        // Record guard usages (positive pattern - auto-release)
        for (module, guard_type) in &self.guard_usages {
            self.stats
                .guard_patterns
                .push(format!("{}::{}", module, guard_type));
        }

        // Detect async spawn/join imbalance
        self.stats.async_spawns = self.async_spawns.len();
        self.stats.async_joins = self.async_joins.len();

        if self.stats.async_spawns > 0 && self.stats.async_joins == 0 {
            self.instances.push(TemporalCouplingInstance {
                pattern: TemporalPattern::RustAsyncSpawnWithoutJoin,
                source: "project-wide".to_string(),
                severity: 0.6,
                description: format!(
                    "Found {} async spawn(s) but no explicit join/await. Tasks may be orphaned.",
                    self.stats.async_spawns
                ),
                suggestion: "Ensure spawned tasks are awaited or their JoinHandles are collected"
                    .to_string(),
            });
            self.stats.total_issues += 1;
        }

        // Detect unsafe allocation patterns
        for (module, operation) in &self.unsafe_allocs {
            self.stats
                .unsafe_allocations
                .push(format!("{}::{}", module, operation));

            // Check for allocation without deallocation
            let has_dealloc = self.unsafe_allocs.iter().any(|(_, op)| {
                op.contains("dealloc") || op.contains("free") || op.contains("drop")
            });

            if operation.contains("alloc") && !has_dealloc {
                self.instances.push(TemporalCouplingInstance {
                    pattern: TemporalPattern::RustUnsafeManualResource {
                        operation: operation.clone(),
                    },
                    source: module.clone(),
                    severity: 0.9,
                    description: format!(
                        "Unsafe allocation '{}' detected without corresponding deallocation",
                        operation
                    ),
                    suggestion: "Ensure manual allocations have corresponding deallocations, or use safe wrappers".to_string(),
                });
                self.stats.total_issues += 1;
            }
        }

        // Record builder patterns
        for (type_name, methods) in &self.builder_types {
            self.stats
                .builder_patterns
                .push(format!("{} ({})", type_name, methods.join(" -> ")));

            if methods.len() >= 3 {
                self.instances.push(TemporalCouplingInstance {
                    pattern: TemporalPattern::RustBuilderPattern {
                        type_name: type_name.clone(),
                        required_methods: methods.clone(),
                    },
                    source: "project-wide".to_string(),
                    severity: 0.3,
                    description: format!(
                        "Builder pattern for '{}' has {} methods that may require specific order",
                        type_name,
                        methods.len()
                    ),
                    suggestion:
                        "Consider using type-state pattern to enforce build order at compile time"
                            .to_string(),
                });
            }
        }
    }

    /// Generate summary report
    pub fn summary(&self) -> String {
        let mut report = String::new();
        report.push_str("## Temporal Coupling Analysis\n\n");

        if self.instances.is_empty()
            && self.stats.drop_impls.is_empty()
            && self.stats.guard_patterns.is_empty()
        {
            report.push_str("No temporal coupling patterns detected.\n");
            return report;
        }

        report.push_str(&format!(
            "**Total Issues**: {}\n\n",
            self.stats.total_issues
        ));

        // Rust-specific positive patterns (RAII)
        if !self.stats.drop_impls.is_empty() || !self.stats.guard_patterns.is_empty() {
            report.push_str("### Rust RAII Patterns (Positive)\n\n");
            report.push_str("These patterns help prevent temporal coupling issues:\n\n");

            if !self.stats.drop_impls.is_empty() {
                report.push_str(&format!(
                    "- **Drop implementations**: {} types with automatic cleanup\n",
                    self.stats.drop_impls.len()
                ));
            }
            if !self.stats.guard_patterns.is_empty() {
                report.push_str(&format!(
                    "- **Guard patterns**: {} auto-release guards used\n",
                    self.stats.guard_patterns.len()
                ));
            }
            report.push('\n');
        }

        // Paired operations
        if !self.stats.paired_operations.is_empty() {
            report.push_str("### Paired Operations\n\n");
            report.push_str("| Operation | Open | Close | Status |\n");
            report.push_str("|-----------|------|-------|--------|\n");

            for (op, stats) in &self.stats.paired_operations {
                let status = if stats.open_count == stats.close_count {
                    "Balanced"
                } else {
                    "Imbalanced"
                };
                report.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    op, stats.open_count, stats.close_count, status
                ));
            }
            report.push('\n');
        }

        // Async spawn/join
        if self.stats.async_spawns > 0 || self.stats.async_joins > 0 {
            report.push_str("### Async Task Management\n\n");
            report.push_str("| Metric | Count |\n");
            report.push_str("|--------|-------|\n");
            report.push_str(&format!("| Spawns | {} |\n", self.stats.async_spawns));
            report.push_str(&format!("| Joins/Awaits | {} |\n", self.stats.async_joins));
            if self.stats.async_spawns > self.stats.async_joins {
                report.push_str("\n**Warning**: More spawns than joins detected.\n");
            }
            report.push('\n');
        }

        // Lifecycle methods
        if !self.stats.lifecycle_methods.is_empty() {
            report.push_str("### Lifecycle Methods\n\n");
            report.push_str("| Phase | Methods |\n");
            report.push_str("|-------|--------|\n");

            let mut phases: Vec<_> = self.stats.lifecycle_methods.iter().collect();
            phases.sort_by_key(|(phase, _)| **phase);

            for (phase, methods) in phases {
                let method_list = if methods.len() > 3 {
                    format!("{}, ... ({} total)", methods[..3].join(", "), methods.len())
                } else {
                    methods.join(", ")
                };
                report.push_str(&format!("| {} | {} |\n", phase.description(), method_list));
            }
            report.push('\n');
        }

        // Unsafe allocations
        if !self.stats.unsafe_allocations.is_empty() {
            report.push_str("### Unsafe Manual Resource Management\n\n");
            report.push_str(
                "**Warning**: Manual memory management detected. Ensure proper cleanup.\n\n",
            );
            for alloc in &self.stats.unsafe_allocations {
                report.push_str(&format!("- `{}`\n", alloc));
            }
            report.push('\n');
        }

        // High severity issues
        let high_severity: Vec<_> = self
            .instances
            .iter()
            .filter(|i| i.severity >= 0.6)
            .collect();

        if !high_severity.is_empty() {
            report.push_str("### Issues Detected\n\n");
            for instance in high_severity {
                let severity_label = if instance.severity >= 0.8 {
                    "Critical"
                } else if instance.severity >= 0.6 {
                    "High"
                } else {
                    "Medium"
                };
                report.push_str(&format!(
                    "- **[{}]** {}\n",
                    severity_label, instance.description
                ));
                report.push_str(&format!("  - Suggestion: {}\n", instance.suggestion));
            }
            report.push('\n');
        }

        report
    }

    /// Get high severity instances
    pub fn high_severity_instances(&self) -> Vec<&TemporalCouplingInstance> {
        self.instances
            .iter()
            .filter(|i| i.severity >= 0.6)
            .collect()
    }
}

/// Analyze source code for temporal patterns
pub fn analyze_temporal_patterns(content: &str, module_name: &str) -> TemporalAnalyzer {
    let mut analyzer = TemporalAnalyzer::new();
    analyzer.set_module(module_name.to_string());

    // Simple pattern matching for method calls and definitions
    // This is a heuristic approach - not full AST parsing

    // Detect function definitions
    let fn_regex = regex_lite::Regex::new(r"fn\s+([a-z_][a-z0-9_]*)\s*[<(]").unwrap();
    for cap in fn_regex.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            analyzer.record_function_def(name.as_str());
        }
    }

    // Detect method calls (simplified)
    let method_regex = regex_lite::Regex::new(r"\.([a-z_][a-z0-9_]*)\s*\(").unwrap();
    for cap in method_regex.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            let name_str = name.as_str();
            analyzer.record_call(name_str);

            // Check for async spawn patterns
            for spawn_pattern in TemporalAnalyzer::RUST_ASYNC_SPAWN_PATTERNS {
                if name_str.contains(spawn_pattern) {
                    analyzer.record_async_spawn(name_str);
                    break;
                }
            }

            // Check for async join patterns
            for join_pattern in TemporalAnalyzer::RUST_ASYNC_JOIN_PATTERNS {
                if name_str.contains(join_pattern) {
                    analyzer.record_async_join(name_str);
                    break;
                }
            }
        }
    }

    // Detect function calls
    let call_regex = regex_lite::Regex::new(r"([a-z_][a-z0-9_]*)\s*\(").unwrap();
    for cap in call_regex.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            let name_str = name.as_str();
            // Skip common keywords
            if ![
                "if", "while", "for", "match", "fn", "let", "return", "Some", "None", "Ok", "Err",
            ]
            .contains(&name_str)
            {
                analyzer.record_call(name_str);
            }
        }
    }

    // Detect Drop impl
    let drop_regex = regex_lite::Regex::new(r"impl\s+Drop\s+for\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
    for cap in drop_regex.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            analyzer.record_drop_impl(name.as_str());
        }
    }

    // Detect guard type usage
    for guard_type in TemporalAnalyzer::RUST_GUARD_TYPES {
        if content.contains(guard_type) {
            analyzer.record_guard_usage(guard_type);
        }
    }

    // Detect unsafe allocation patterns
    for pattern in TemporalAnalyzer::RUST_UNSAFE_ALLOC_PATTERNS {
        if content.contains(pattern) {
            analyzer.record_unsafe_alloc(pattern);
        }
    }

    analyzer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paired_operation_detection() {
        let mut analyzer = TemporalAnalyzer::new();
        analyzer.set_module("test".to_string());

        // Simulate open calls
        analyzer.record_call("open");
        analyzer.record_call("open");
        // Simulate close calls (fewer)
        analyzer.record_call("close");

        analyzer.analyze();

        assert_eq!(
            analyzer
                .stats
                .paired_operations
                .get("open/close")
                .unwrap()
                .open_count,
            2
        );
        assert_eq!(
            analyzer
                .stats
                .paired_operations
                .get("open/close")
                .unwrap()
                .close_count,
            1
        );
        assert!(analyzer.stats.total_issues > 0);
    }

    #[test]
    fn test_lifecycle_detection() {
        let mut analyzer = TemporalAnalyzer::new();
        analyzer.set_module("test".to_string());

        analyzer.record_function_def("initialize");
        analyzer.record_function_def("start_server");
        analyzer.record_function_def("process_request");

        analyzer.analyze();

        assert!(
            analyzer
                .stats
                .lifecycle_methods
                .contains_key(&LifecyclePhase::Initialize)
        );
        assert!(
            analyzer
                .stats
                .lifecycle_methods
                .contains_key(&LifecyclePhase::Start)
        );
    }

    #[test]
    fn test_state_check_detection() {
        let mut analyzer = TemporalAnalyzer::new();
        analyzer.set_module("test".to_string());

        analyzer.record_function_def("is_initialized");
        analyzer.record_function_def("check_connected");

        analyzer.analyze();

        assert!(!analyzer.stats.state_checks.is_empty());
    }

    #[test]
    fn test_balanced_operations() {
        let mut analyzer = TemporalAnalyzer::new();
        analyzer.set_module("test".to_string());

        analyzer.record_call("lock");
        analyzer.record_call("unlock");
        analyzer.record_call("lock");
        analyzer.record_call("unlock");

        analyzer.analyze();

        let stats = analyzer.stats.paired_operations.get("lock/unlock").unwrap();
        assert_eq!(stats.open_count, stats.close_count);
    }

    #[test]
    fn test_analyze_temporal_patterns() {
        let code = r#"
            fn initialize(&mut self) {
                self.ready = true;
            }

            fn process(&mut self) {
                if self.is_initialized() {
                    self.handle_request();
                }
            }

            fn cleanup(&mut self) {
                self.close();
            }
        "#;

        let analyzer = analyze_temporal_patterns(code, "test_module");
        assert!(!analyzer.function_defs.is_empty());
    }
}
