//! Validation regex generated from possible values.

use pm_config::{Config, ConfigOption, PossibleValue, OPT_TYPE_STRING};
use serde_json::{json, Value};

fn path_value(path: &str) -> PossibleValue {
    PossibleValue {
        name: path.to_string(),
        description: String::new(),
        value: Value::String(path.to_string()),
    }
}

#[test]
fn windows_paths_in_possible_values_are_escaped() {
    let config = Config::new();
    config
        .register(
            ConfigOption::new("Bin", "test/bin", "d", OPT_TYPE_STRING, json!(""))
                .with_possible_values(vec![
                    path_value(""),
                    path_value(r"C:\Program Files\v2rayN\bin\xray\xray.exe"),
                    path_value("xray.exe"),
                ]),
        )
        .expect("register must not fail on backslashes");

    config
        .set_config_option("test/bin", json!(r"C:\Program Files\v2rayN\bin\xray\xray.exe"))
        .unwrap();
    config.set_config_option("test/bin", json!("xray.exe")).unwrap();
    // The dot is literal now, so a look-alike value is rejected.
    assert!(config.set_config_option("test/bin", json!("xrayXexe")).is_err());
}
