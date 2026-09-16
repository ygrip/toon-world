use std::path::PathBuf;

use clap::Parser;
use toon_world::cli::{Args, OutputFormat};

#[test]
fn defaults_to_identity_query_and_toon_output() {
    let args = Args::try_parse_from(["toon-world"]).unwrap();

    assert_eq!(args.file, None);
    assert_eq!(args.query, ".");
    assert_eq!(args.to, OutputFormat::Toon);
}

#[test]
fn parses_file_query_and_output_format() {
    let args = Args::try_parse_from([
        "toon-world",
        "data.json",
        "-q",
        ".users",
        "--to",
        "json",
    ])
    .unwrap();

    assert_eq!(args.file, Some(PathBuf::from("data.json")));
    assert_eq!(args.query, ".users");
    assert_eq!(args.to, OutputFormat::Json);
}

#[test]
fn dash_is_preserved_as_explicit_stdin_path() {
    let args = Args::try_parse_from(["toon-world", "-", "--to", "text"]).unwrap();

    assert_eq!(args.file, Some(PathBuf::from("-")));
    assert_eq!(args.to, OutputFormat::Text);
}

#[test]
fn rejects_unknown_output_format() {
    let error = Args::try_parse_from(["toon-world", "--to", "yaml"])
        .expect_err("unsupported output format must be rejected");

    assert!(error.to_string().contains("invalid value"));
}
