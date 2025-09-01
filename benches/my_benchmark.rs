use criterion::{Criterion, criterion_group, criterion_main};
use rand::{SeedableRng, rngs::SmallRng};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use validatrix::Validate;

#[derive(Debug, Serialize, Deserialize)]
struct MyStruct {
    is_valid: bool,
    children: Vec<MyStruct>,
}

impl MyStruct {
    fn count(&self) -> usize {
        let subcount: usize = self.children.iter().map(|c| c.count()).sum();
        subcount + 1
    }

    fn count_valid(&self) -> usize {
        let subcount: usize = self.children.iter().map(|c| c.count_valid()).sum();
        if self.is_valid {
            subcount + 1
        } else {
            subcount
        }
    }
}

impl Validate for MyStruct {
    fn validate_inner(&self, accum: &mut validatrix::Accumulator) -> usize {
        let orig = accum.len();
        if !self.is_valid {
            accum.add_failure("not valid".into(), &["is_valid".into()]);
        }
        accum.validate_iter("children", &self.children);
        accum.len() - orig
    }
}

fn make_struct(
    n_children: usize,
    depth: usize,
    valid_chance: f64,
    rng: &mut impl rand::Rng,
) -> MyStruct {
    let children = if depth == 0 {
        vec![]
    } else {
        std::iter::repeat_with(|| make_struct(n_children, depth - 1, valid_chance, rng))
            .take(n_children)
            .collect()
    };
    MyStruct {
        is_valid: rng.random_bool(valid_chance),
        children,
    }
}

fn standard_struct() -> MyStruct {
    let mut rng = SmallRng::seed_from_u64(1991);
    let s = make_struct(3, 10, 0.01, &mut rng);
    println!("Struct with {} nodes, {} valid", s.count(), s.count_valid());
    s
}

fn ser_benchmark(c: &mut Criterion) {
    let s = standard_struct();
    c.bench_function("serialize", |b| {
        b.iter(|| {
            let _ser = serde_json::to_string(black_box(&s)).unwrap();
        })
    });
}

fn de_benchmark(c: &mut Criterion) {
    let s = standard_struct();
    let json = serde_json::to_string(&s).unwrap();
    c.bench_function("deserialize", |b| {
        b.iter(|| {
            let _mys: MyStruct = serde_json::from_str(black_box(&json)).unwrap();
        })
    });
}

fn validate_benchmark(c: &mut Criterion) {
    let s = standard_struct();
    c.bench_function("validate", |b| {
        b.iter(|| {
            let _res = black_box(&s).validate();
        })
    });
}

criterion_group!(benches, ser_benchmark, de_benchmark, validate_benchmark);
criterion_main!(benches);
