use assert_cmd::Command;

#[test]
fn raw_markdown_data_supports_section_queries() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--from",
            "markdown",
            "--data",
            "# Demo\n\n## Usage\n\nRun it.\n",
            "-q",
            r#"section("Usage") | .heading"#,
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Usage\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn raw_markdown_data_can_extract_fenced_code() {
    let output = Command::cargo_bin("toon-world")
        .unwrap()
        .args([
            "--from",
            "markdown",
            "--data",
            "# Demo\n\n## Usage\n\n```bash\ncargo run\n```\n",
            "-q",
            r#"section("Usage") | code("bash") | .text"#,
            "--to",
            "text",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "cargo run\n");
}
