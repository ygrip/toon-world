use toon_world::sparse;

#[path = "../benches/support/mod.rs"]
mod support;

#[test]
fn generated_production_matrix_round_trips() {
    let fixtures = support::fixtures();
    assert_eq!(fixtures.len(), 36);
    for fixture in fixtures {
        let rendered = sparse::encode(&fixture.value)
            .unwrap_or_else(|error| panic!("{} failed to encode: {error}", fixture.name))
            .unwrap_or_else(|| panic!("{} declined sparse encoding", fixture.name));
        assert_eq!(
            sparse::decode(&rendered).unwrap(),
            Some(fixture.value),
            "{} did not round trip",
            fixture.name
        );
    }
}
