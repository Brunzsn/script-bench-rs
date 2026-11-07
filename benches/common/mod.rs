use std::time::Duration;

use criterion::Criterion;

/// Shared Criterion configuration for every benchmark target.
pub fn config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10))
}
