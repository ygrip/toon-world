use serde_json::json;

pub fn render(input_bytes: usize, output_bytes: usize) -> String {
    let reduction_percent = if input_bytes == 0 {
        None
    } else {
        let reduction = (1.0 - output_bytes as f64 / input_bytes as f64) * 100.0;
        Some(round_two(reduction))
    };

    json!({
        "input_bytes": input_bytes,
        "output_bytes": output_bytes,
        "reduction_percent": reduction_percent,
    })
    .to_string()
}

fn round_two(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
