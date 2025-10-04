/// Custom filter for rounding floats
pub fn round(value: &f64) -> askama::Result<String> {
    Ok(format!("{:.0}", value))
}
