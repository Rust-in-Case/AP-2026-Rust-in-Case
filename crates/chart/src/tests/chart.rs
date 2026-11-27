#[cfg(test)]
use crate::chart_ric::RicChart;
use contractors::conformance::chart::{RefChart, generate};

#[test]
fn conformance() {
    generate::<RefChart, RicChart>();
}
