//! Benchmarks for cargo-coupling analysis
//!
//! This benchmark uses the cargo-coupling project itself as the test subject,
//! ensuring it works in any environment without hardcoded paths.

use cargo_coupling::{analyze_project, analyze_project_balance};
use criterion::{Criterion, criterion_group, criterion_main};
use std::path::PathBuf;

/// Get the src directory of this project for benchmarking
fn get_project_src_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("src")
}

fn benchmark_analyze_project(c: &mut Criterion) {
    let src_dir = get_project_src_dir();

    if !src_dir.exists() {
        eprintln!("Warning: src directory not found at {:?}", src_dir);
        return;
    }

    c.bench_function("analyze_project", |b| {
        b.iter(|| {
            let _ = analyze_project(&src_dir);
        })
    });
}

fn benchmark_analyze_balance(c: &mut Criterion) {
    let src_dir = get_project_src_dir();

    if !src_dir.exists() {
        return;
    }

    // First get project metrics
    let metrics = analyze_project(&src_dir).expect("Failed to analyze project");

    c.bench_function("analyze_balance", |b| {
        b.iter(|| {
            let _ = analyze_project_balance(&metrics);
        })
    });
}

fn benchmark_full_analysis(c: &mut Criterion) {
    let src_dir = get_project_src_dir();

    if !src_dir.exists() {
        return;
    }

    c.bench_function("full_analysis", |b| {
        b.iter(|| {
            let metrics = analyze_project(&src_dir).expect("Failed to analyze");
            let _ = analyze_project_balance(&metrics);
        })
    });
}

criterion_group!(
    benches,
    benchmark_analyze_project,
    benchmark_analyze_balance,
    benchmark_full_analysis
);
criterion_main!(benches);
