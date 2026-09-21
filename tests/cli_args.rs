use std::path::PathBuf;

use clap::Parser;
use toon_world::cli::{Args, InputFormat, OutputFormat};

#[test]
fn defaults_to_json_autodetection_identity_query_and_toon_output() {
    let args = Args::try_parse_from(["toon-world"]).unwrap();

    assert_eq!(args.file, None);
    assert_eq!(args.data, None);
    assert_eq!(args.from, None);
    assert!(!args.quiet);
    assert!(!args.warnings_as_errors);
    assert_eq!(args.query, ".");
    assert_eq!(args.to, OutputFormat::Toon);
}

#[test]
fn parses_explicit_input_and_output_formats() {
    let args = Args::try_parse_from([
        "toon-world",
        "payload.txt",
        "--from",
        "csv",
        "-q",
        ".[0]",
        "--to",
        "json",
    ])
    .unwrap();

    assert_eq!(args.file, Some(PathBuf::from("payload.txt")));
    assert_eq!(args.data, None);
    assert_eq!(args.from, Some(InputFormat::Csv));
    assert_eq!(args.query, ".[0]");
    assert_eq!(args.to, OutputFormat::Json);
}

#[test]
fn parses_raw_data_with_explicit_format() {
    let args = Args::try_parse_from([
        "toon-world",
        "--data",
        "id,name\n1,Ada\n",
        "--from",
        "csv",
        "--quiet",
    ])
    .unwrap();

    assert_eq!(args.file, None);
    assert_eq!(args.data.as_deref(), Some("id,name\n1,Ada\n"));
    assert_eq!(args.from, Some(InputFormat::Csv));
    assert!(args.quiet);
}

#[test]
fn dash_is_preserved_as_explicit_stdin_path() {
    let args = Args::try_parse_from(["toon-world", "-", "--from", "yaml", "--to", "text"]).unwrap();

    assert_eq!(args.file, Some(PathBuf::from("-")));
    assert_eq!(args.from, Some(InputFormat::Yaml));
    assert_eq!(args.to, OutputFormat::Text);
}

#[test]
fn rejects_file_and_raw_data_together() {
    let error = Args::try_parse_from(["toon-world", "data.json", "--data", r#"{"name":"Ada"}"#])
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
fn rejects_unknown_input_format() {
    let error = Args::try_parse_from(["toon-world", "--from", "ini"])
        .expect_err("unsupported input format must be rejected");

    assert!(error.to_string().contains("invalid value"));
}

#[test]
fn rejects_unknown_output_format() {
    let error = Args::try_parse_from(["toon-world", "--to", "yaml"])
        .expect_err("unsupported output format must be rejected");

    assert!(error.to_string().contains("invalid value"));
}
