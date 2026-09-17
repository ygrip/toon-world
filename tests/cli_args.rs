use std::path::PathBuf;

use clap::Parser;
use toon_world::cli::{Args, OutputFormat};

#[test]
fn defaults_to_identity_query_and_toon_output() {
    let args = Args::try_parse_from(["toon-world"]).unwrap();

    assert_eq!(args.file, None);
    assert_eq!(args.data, None);
    assert!(!args.quiet);
    assert!(!args.warnings_as_errors);
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
    assert_eq!(args.data, None);
    assert_eq!(args.query, ".users");
    assert_eq!(args.to, OutputFormat::Json);
}

#[test]
fn parses_raw_data_and_warning_flags() {
    let args = Args::try_parse_from([
        "toon-world",
        "--data",
        r#"{"name":"Ada"}"#,
        "--quiet",
        "--to",
        "json",
    ])
    .unwrap();

    assert_eq!(args.file, None);
    assert_eq!(args.data.as_deref(), Some(r#"{"name":"Ada"}"#));
    assert!(args.quiet);
    assert!(!args.warnings_as_errors);
}

#[test]
fn dash_is_preserved_as_explicit_stdin_path() {
    let args = Args::try_parse_from(["toon-world", "-", "--to", "text"]).unwrap();

    assert_eq!(args.file, Some(PathBuf::from("-")));
    assert_eq!(args.to, OutputFormat::Text);
}

#[test]
fn rejects_file_and_raw_data_together() {
    let error = Args::try_parse_from([
        "toon-world",
        "data.json",
        "--data",
        r#"{"name":"Ada"}"#,
    ])
    .expect_err("file and --data must be mutually exclusive");

    assert!(error.to_string().contains("cannot be used with"));
}

#[test]
fn rejects_quiet_and_warnings_as_errors_together() {
    let error = Args::try_parse_from(["toon-world", "--quiet", "--warnings-as-errors"])
        .expect_err("warning policies must be mutually exclusive");

    assert!(error.to_string().contains("cannot be used with"));
}

#[test]
fn rejects_unknown_output_format() {
    let error = Args::try_parse_from(["toon-world", "--to", "yaml"])
        .expect_err("unsupported output format must be rejected");

    assert!(error.to_string().contains("invalid value"));
}
