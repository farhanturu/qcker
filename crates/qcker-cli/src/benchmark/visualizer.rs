use crate::benchmark::metrics::BenchmarkSuite;

pub struct BenchmarkVisualizer;

impl BenchmarkVisualizer {
    pub fn print_ascii_chart(suite: &BenchmarkSuite) {
        println!("\n{}", "═".repeat(60));
        println!("  ASCII CHART: Duration (ms)");
        println!("{}", "═".repeat(60));

        let max_duration = suite.results.iter()
            .map(|r| r.duration.as_millis())
            .max()
            .unwrap_or(0) as f64;

        for result in &suite.results {
            let bar_len = (result.duration.as_millis() as f64 / max_duration * 40.0) as usize;
            let bar = "█".repeat(bar_len);
            println!("  {:<25} │{} {:.0}ms", result.name, bar, result.duration.as_millis());
        }
        println!("{}", "═".repeat(60));
    }

    pub fn print_ascii_comparison(suite_a: &BenchmarkSuite, suite_b: &BenchmarkSuite) {
        println!("\n{}", "═".repeat(70));
        println!("  ASCII COMPARISON: {} vs {}", suite_a.name, suite_b.name);
        println!("{}", "═".repeat(70));

        let max_val = f64::max(suite_a.avg_duration().as_millis() as f64,
                               suite_b.avg_duration().as_millis() as f64);

        let a_bar = (suite_a.avg_duration().as_millis() as f64 / max_val * 30.0) as usize;
        let b_bar = (suite_b.avg_duration().as_millis() as f64 / max_val * 30.0) as usize;

        println!("  {:<15} │{} {:.0}ms", suite_a.name, "█".repeat(a_bar), suite_a.avg_duration().as_millis());
        println!("  {:<15} │{} {:.0}ms", suite_b.name, "█".repeat(b_bar), suite_b.avg_duration().as_millis());
        println!("{}", "═".repeat(70));
    }
}
