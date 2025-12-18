//! Git history analysis for volatility measurement
//!
//! Analyzes git log to determine how frequently files change.
//! Optimized for large repositories using streaming and git path filtering.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use thiserror::Error;

use crate::metrics::Volatility;

/// Errors that can occur during volatility analysis
#[derive(Error, Debug)]
pub enum VolatilityError {
    #[error("Failed to execute git command: {0}")]
    GitCommand(#[from] std::io::Error),

    #[error("Invalid UTF-8 in git output: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Not a git repository")]
    NotGitRepo,
}

/// Volatility analyzer using git history
#[derive(Debug, Default)]
pub struct VolatilityAnalyzer {
    /// File path -> change count
    pub file_changes: HashMap<String, usize>,
    /// Analysis period in months
    pub period_months: usize,
}

impl VolatilityAnalyzer {
    /// Create a new volatility analyzer
    pub fn new(period_months: usize) -> Self {
        Self {
            file_changes: HashMap::new(),
            period_months,
        }
    }

    /// Analyze git history for a repository (optimized version)
    ///
    /// Optimizations applied:
    /// 1. Use `-- "*.rs"` to filter .rs files at git level
    /// 2. Use streaming with BufReader instead of loading all into memory
    /// 3. Use `--diff-filter=AMRC` to skip deleted files
    pub fn analyze(&mut self, repo_path: &Path) -> Result<(), VolatilityError> {
        // Check if it's a git repo
        let git_check = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(repo_path)
            .stderr(Stdio::null())
            .output()?;

        if !git_check.status.success() {
            return Err(VolatilityError::NotGitRepo);
        }

        // Optimized: use --diff-filter and path spec to reduce output
        // --diff-filter=AMRC: Added, Modified, Renamed, Copied (skip Deleted)
        let mut child = Command::new("git")
            .args([
                "log",
                "--pretty=format:",
                "--name-only",
                "--diff-filter=AMRC",
                &format!("--since={} months ago", self.period_months),
                "--",
                "*.rs",
            ])
            .current_dir(repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        // Stream processing with BufReader
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::with_capacity(64 * 1024, stdout); // 64KB buffer

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => continue,
                };

                let line = line.trim();
                if !line.is_empty() && line.ends_with(".rs") {
                    *self.file_changes.entry(line.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Wait for git to finish
        let _ = child.wait();

        Ok(())
    }

    /// Get volatility level for a file
    pub fn get_volatility(&self, file_path: &str) -> Volatility {
        let count = self.file_changes.get(file_path).copied().unwrap_or(0);
        Volatility::from_count(count)
    }

    /// Get change count for a file
    pub fn get_change_count(&self, file_path: &str) -> usize {
        self.file_changes.get(file_path).copied().unwrap_or(0)
    }

    /// Get all high volatility files
    pub fn high_volatility_files(&self) -> Vec<(&String, usize)> {
        self.file_changes
            .iter()
            .filter(|&(_, count)| *count > 10)
            .map(|(path, count)| (path, *count))
            .collect()
    }

    /// Get volatility statistics
    pub fn statistics(&self) -> VolatilityStats {
        if self.file_changes.is_empty() {
            return VolatilityStats::default();
        }

        let counts: Vec<usize> = self.file_changes.values().copied().collect();
        let total: usize = counts.iter().sum();
        let max = counts.iter().max().copied().unwrap_or(0);
        let min = counts.iter().min().copied().unwrap_or(0);
        let avg = total as f64 / counts.len() as f64;

        let low_count = counts.iter().filter(|&&c| c <= 2).count();
        let medium_count = counts.iter().filter(|&&c| c > 2 && c <= 10).count();
        let high_count = counts.iter().filter(|&&c| c > 10).count();

        VolatilityStats {
            total_files: counts.len(),
            total_changes: total,
            max_changes: max,
            min_changes: min,
            avg_changes: avg,
            low_volatility_count: low_count,
            medium_volatility_count: medium_count,
            high_volatility_count: high_count,
        }
    }
}

/// Statistics about volatility across the project
#[derive(Debug, Default)]
pub struct VolatilityStats {
    pub total_files: usize,
    pub total_changes: usize,
    pub max_changes: usize,
    pub min_changes: usize,
    pub avg_changes: f64,
    pub low_volatility_count: usize,
    pub medium_volatility_count: usize,
    pub high_volatility_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_classification() {
        let mut analyzer = VolatilityAnalyzer::new(6);
        analyzer.file_changes.insert("stable.rs".to_string(), 1);
        analyzer.file_changes.insert("moderate.rs".to_string(), 5);
        analyzer.file_changes.insert("volatile.rs".to_string(), 15);

        assert_eq!(analyzer.get_volatility("stable.rs"), Volatility::Low);
        assert_eq!(analyzer.get_volatility("moderate.rs"), Volatility::Medium);
        assert_eq!(analyzer.get_volatility("volatile.rs"), Volatility::High);
        assert_eq!(analyzer.get_volatility("unknown.rs"), Volatility::Low);
    }

    #[test]
    fn test_high_volatility_files() {
        let mut analyzer = VolatilityAnalyzer::new(6);
        analyzer.file_changes.insert("stable.rs".to_string(), 2);
        analyzer.file_changes.insert("volatile.rs".to_string(), 15);
        analyzer
            .file_changes
            .insert("very_volatile.rs".to_string(), 25);

        let high_vol = analyzer.high_volatility_files();
        assert_eq!(high_vol.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut analyzer = VolatilityAnalyzer::new(6);
        analyzer.file_changes.insert("a.rs".to_string(), 1);
        analyzer.file_changes.insert("b.rs".to_string(), 5);
        analyzer.file_changes.insert("c.rs".to_string(), 15);

        let stats = analyzer.statistics();
        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_changes, 21);
        assert_eq!(stats.max_changes, 15);
        assert_eq!(stats.min_changes, 1);
        assert_eq!(stats.low_volatility_count, 1);
        assert_eq!(stats.medium_volatility_count, 1);
        assert_eq!(stats.high_volatility_count, 1);
    }
}
