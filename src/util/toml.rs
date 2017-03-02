use toml;

/// Get a user-facing string out of a `toml::Value`.
///
/// `toml::Value::to_string()` puts quotes around the string, which makes it
/// unsuitable for this purpose.
pub fn value_to_string(value: &toml::Value) -> String {
    match *value {
        toml::Value::String(ref s) => s.clone(),
        // For other cases, fall back to to_string()
        _ => value.to_string(),
    }
}

#[test]
fn test_value_to_string() {
    assert_eq!(value_to_string(&toml::Value::String("hello".into())),
               "hello");
    assert_eq!(value_to_string(&toml::Value::Integer(763)), "763");
}
