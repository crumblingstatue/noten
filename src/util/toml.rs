use toml;

pub fn parser_error_to_string(parser: &toml::Parser) -> String {
    let mut msg = String::new();
    for e in &parser.errors {
        let (lo_line, lo_col) = parser.to_linecol(e.lo);
        let (hi_line, hi_col) = parser.to_linecol(e.hi);
        msg.push_str(&format!("{}:{} -> {}:{} : {}\n",
                              lo_line + 1,
                              lo_col + 1,
                              hi_line + 1,
                              hi_col + 1,
                              e.desc));
    }
    msg
}

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
    assert_eq!(value_to_string(&toml::Value::String("hello".into())), "hello");
    assert_eq!(value_to_string(&toml::Value::Integer(763)), "763");
}
