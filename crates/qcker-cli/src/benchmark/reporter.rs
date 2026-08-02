use crate::benchmark::metrics::{BenchmarkResult, BenchmarkSuite};
use std::fmt;

pub struct BenchmarkReporter;

impl BenchmarkReporter {
    pub fn print_header(suite: &BenchmarkSuite) {
        println!("\n{}", "═".repeat(60));
        println!("  BENCHMARK SUITE: {}", suite.name);
        println!("  Total Duration: {:.2}s", suite.total_duration().as_secs_f64());
        println!("{}", "═".repeat(60));
    }

    pub fn print_summary(suite: &BenchmarkSuite) {
        println!("\n{}", "─".repeat(60));
        println!("  SUMMARY");
        println!("{}", "─".repeat(60));
        println!("  Tests Run:      {}", suite.results.len());
        println!("  Success Rate:   {:.1}%", suite.success_rate());
        println!("  Avg Duration:   {:.3}s", suite.avg_duration().as_secs_f64());
        if let Some(min) = suite.min_duration() {
            println!("  Min Duration:   {:.3}s", min.as_secs_f64());
        }
        if let Some(max) = suite.max_duration() {
            println!("  Max Duration:   {:.3}s", max.as_secs_f64());
        }
        println!("  Avg Memory:     {:.2} MB", suite.avg_memory_mb());
        println!("  Avg CPU:        {:.1}%", suite.avg_cpu());
        println!("{}", "─".repeat(60));
    }

    pub fn print_results(suite: &BenchmarkSuite) {
        println!("\n{}", "─".repeat(60));
        println!("  DETAILED RESULTS");
        println!("{}", "─".repeat(60));
        println!("{:<30} {:<12} {:<12} {:<10} {}", "Test Name", "Duration", "Memory(MB)", "CPU%", "Status");
        println!("{}", "─".repeat(60));

        for result in &suite.results {
            let status = match result.status.as_str() {
                "success" => "OK",
                "failed" => "FAIL",
                _ => "WARN",
            };
            println!(
                "{:<30} {:<12.3}s {:<12.2} {:<10.1} {}",
                result.name, result.duration.as_secs_f64(),
                result.memory_usage_mb, result.cpu_percent, status
            );
        }
        println!("{}", "─".repeat(60));
    }

    pub fn print_comparison(suite_a: &BenchmarkSuite, suite_b: &BenchmarkSuite) {
        println!("\n{}", "═".repeat(60));
        println!("  COMPARISON: {} vs {}", suite_a.name, suite_b.name);
        println!("{}", "═".repeat(60));

        let avg_a = suite_a.avg_duration().as_secs_f64();
        let avg_b = suite_b.avg_duration().as_secs_f64();

        println!("\n  {:<20} {:<15} {:<15}", "Metric", suite_a.name, suite_b.name);
        println!("  {}", "─".repeat(50));
        println!("  {:<20} {:<15.3}s {:<15.3}s", "Avg Duration", avg_a, avg_b);
        println!("  {:<20} {:<15.2}MB {:<15.2}MB", "Avg Memory", suite_a.avg_memory_mb(), suite_b.avg_memory_mb());
        println!("  {:<20} {:<15.1}% {:<15.1}%", "Avg CPU", suite_a.avg_cpu(), suite_b.avg_cpu());
        println!("  {:<20} {:<15.1}% {:<15.1}%", "Success Rate", suite_a.success_rate(), suite_b.success_rate());

        let winner = if avg_a < avg_b { &suite_a.name } else { &suite_b.name };
        println!("\n  Winner: {}", winner);
        println!("{}", "═".repeat(60));
    }
}

impl fmt::Display for BenchmarkReporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BenchmarkReporter")
    }
}
