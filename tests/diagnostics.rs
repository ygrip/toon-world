use toon_world::diagnostics::{resolve_warnings, Warning};

#[test]
fn warning_has_stable_category_format() {
    let warning = Warning::new("input", "unknown extension '.txt'; assuming JSON");

    assert_eq!(
        warning.to_string(),
        "warning[input]: unknown extension '.txt'; assuming JSON"
    );
}

#[test]
fn normal_warning_policy_renders_all_warnings() {
    let warnings = vec![
        Warning::new("input", "first"),
        Warning::new("format", "second"),
    ];

    let rendered = resolve_warnings(&warnings, false, false).unwrap();

    assert_eq!(rendered, "warning[input]: first\nwarning[format]: second");
}

#[test]
fn quiet_policy_suppresses_warnings() {
    let warnings = vec![Warning::new("input", "ignored")];

    let rendered = resolve_warnings(&warnings, true, false).unwrap();

    assert!(rendered.is_empty());
}

#[test]
fn warnings_as_errors_escalates_without_losing_category() {
    let warnings = vec![Warning::new(
        "input",
        "unknown extension '.txt'; assuming JSON",
    )];

    let error = resolve_warnings(&warnings, false, true)
        .expect_err("warnings-as-errors must fail")
        .to_string();

    assert!(error.contains("error[warning-as-error]"));
    assert!(error.contains("warning[input]"));
    assert!(error.contains("assuming JSON"));
}

#[test]
fn no_warnings_produces_no_diagnostic_output_for_any_policy() {
    assert_eq!(resolve_warnings(&[], false, false).unwrap(), "");
    assert_eq!(resolve_warnings(&[], true, false).unwrap(), "");
    assert_eq!(resolve_warnings(&[], false, true).unwrap(), "");
}
