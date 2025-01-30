use std::{fs, path::Path};

fn parse_memory_size(size: String) -> Option<u64> {
    let size = size.trim();
    let Some(split_n) = size.find(" ") else {
        eprintln!("Expected a space in the size str: {size}");
        return None;
    };

    let (num_str, unit) = size.trim_matches('"').split_at(split_n);

    let Ok(num) = num_str.trim().parse::<u64>() else {
        eprintln!("Invalid number format: {size}, '{num_str}', '{unit}'");
        return None;
    };

    let unit = unit.trim().to_lowercase();

    let multiplier: usize = match unit.as_str() {
        "kb" => 1_000,
        "kib" => 1_024,
        "mb" => 1_000 * 1_000,
        "mib" => 1_024 * 1_024,
        "gb" => 1_000 * 1_000 * 1_000,
        "gib" => 1_024 * 1_024 * 1_024,
        "tb" => 1_000 * 1_000 * 1_000 * 1_000,
        "tib" => 1_024 * 1_024 * 1_024 * 1_024,
        _ => {
            eprintln!("Unknown unit");
            return None;
        }
    };

    Some((num as f64 * multiplier as f64).round() as u64)
}

fn main() {
    let rocket_toml_path = Path::new("..").join("Rocket.toml");

    let Ok(contents) = fs::read_to_string(&rocket_toml_path) else {
        panic!("Could not find Rocket.toml config file in {rocket_toml_path:?}");
    };

    let value: toml::Value = contents.parse().expect("Unable to parse TOML");

    let max_upload_size = value
        .get("default")
        .and_then(|defaults| defaults.get("limits"))
        .and_then(|limits| limits.get("file"))
        .map(|value| value.to_string())
        .and_then(parse_memory_size)
        .expect("file upload size not found in Rocket.toml");

    println!("cargo:rustc-env=MAX_UPLOAD_SIZE={}", max_upload_size);
}
