use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use investment_tracker::core::{Investment, InvestmentType, Storage};
use tempfile::tempdir;

fn make_investment(i: usize) -> Investment {
    Investment::new(
        String::new(),
        InvestmentType::Stock,
        format!("Company {i}"),
        Some(format!("SYM{i}")),
        1000.0 + i as f64,
        "2024-01-15".to_string(),
        Some(1100.0 + i as f64),
        None,
        None,
        None,
        None,
    )
    .unwrap()
}

/// Benchmark bulk-adding N investments.
fn bench_add_investments(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_investments");

    for n in [100_usize, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let dir = tempdir().unwrap();
                let path = dir.path().join("investments.json");
                let storage = Storage::new(path);
                let batch: Vec<Investment> = (0..n).map(make_investment).collect();
                storage.add_investments(batch).unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark loading (deserialising) N investments from disk.
fn bench_load_investments(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_investments");

    for n in [100_usize, 1_000, 10_000] {
        // Prepare the data file once outside the timed loop.
        let dir = tempdir().unwrap();
        let path = dir.path().join("investments.json");
        let storage = Storage::new(path.clone());
        let batch: Vec<Investment> = (0..n).map(make_investment).collect();
        storage.add_investments(batch).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            let s = Storage::new(path.clone());
            b.iter(|| {
                s.get_all_investments().unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_add_investments, bench_load_investments);
criterion_main!(benches);
