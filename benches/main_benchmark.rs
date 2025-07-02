mod graph_csm_benches;
mod graph_dyn_benches;

use crate::graph_csm_benches::csm_benches::csm_benches;
use crate::graph_dyn_benches::dyn_benches::dyn_benches;
use criterion::criterion_main;

criterion_main!(csm_benches, dyn_benches);
