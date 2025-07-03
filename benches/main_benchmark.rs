mod graph_csm_benches;
mod graph_dyn_benches;

// Conditionally import the parallel benchmarks only when the feature is enabled.
#[cfg(feature = "parallel")]
use crate::graph_csm_benches::csm_par_benches::csm_par_benches;

use crate::graph_csm_benches::csm_benches::csm_benches;
use crate::graph_dyn_benches::dyn_benches::dyn_benches;
use criterion::criterion_main;

// This version runs when the "parallel" feature is NOT enabled.
#[cfg(not(feature = "parallel"))]
criterion_main!(csm_benches, dyn_benches);

// This version runs when the "parallel" feature IS enabled, including the new suite.
#[cfg(feature = "parallel")]
criterion_main!(csm_benches, csm_par_benches, dyn_benches);
