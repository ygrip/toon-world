use std::alloc::System;

use stats_alloc::{Region, Stats, StatsAlloc, INSTRUMENTED_SYSTEM};
use toon_world::{output, sparse};

#[path = "../benches/support/mod.rs"]
mod support;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
    println!("| fixture | path | allocations | allocated bytes |");
    println!("| --- | --- | ---: | ---: |");
    for fixture in support::fixtures() {
        report(&fixture.name, "standard", || {
            toon_format::encode_default(&fixture.value).expect("standard TOON must encode")
        });
        report(&fixture.name, "sparse", || {
            sparse::encode(&fixture.value).expect("sparse encoding must not fail")
        });
        report(&fixture.name, "compact", || {
            let standard =
                toon_format::encode_default(&fixture.value).expect("standard TOON must encode");
            output::select_compact_toon(
                standard,
                sparse::encode(&fixture.value).expect("sparse encoding must not fail"),
            )
            .into_rendered()
        });
    }
}

fn report<T>(fixture: &str, path: &str, operation: impl FnOnce() -> T) {
    let region = Region::new(&GLOBAL);
    let _result = operation();
    let Stats {
        allocations,
        bytes_allocated,
        ..
    } = region.change();
    println!("| {fixture} | {path} | {allocations} | {bytes_allocated} |");
}
