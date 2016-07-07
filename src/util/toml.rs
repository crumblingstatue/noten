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
    assert_eq!(value_to_string(&toml::Value::String("hello".into())),
               "hello");
    assert_eq!(value_to_string(&toml::Value::Integer(763)), "763");
}

pub struct Extractor {
    table: toml::Table,
    path: String,
}

pub trait TryFromValue: Sized {
    /// Try to extract this type from `value`.
    ///
    /// On error, returns the type string of the value.
    fn from_value(value: &toml::Value) -> Result<Self, &'static str>;
    fn type_str() -> &'static str;
}

impl TryFromValue for String {
    fn from_value(value: &toml::Value) -> Result<Self, &'static str> {
        match *value {
            toml::Value::String(ref s) => Ok(s.clone()),
            _ => Err(value.type_str()),
        }
    }
    fn type_str() -> &'static str {
        "string"
    }
}

impl TryFromValue for toml::Table {
    fn from_value(value: &toml::Value) -> Result<Self, &'static str> {
        match *value {
            toml::Value::Table(ref t) => Ok(t.clone()),
            _ => Err(value.type_str()),
        }
    }
    fn type_str() -> &'static str {
        "table"
    }
}

impl TryFromValue for i64 {
    fn from_value(value: &toml::Value) -> Result<Self, &'static str> {
        match *value {
            toml::Value::Integer(i) => Ok(i),
            _ => Err(value.type_str()),
        }
    }
    fn type_str() -> &'static str {
        "integer"
    }
}

impl Extractor {
    pub fn new(table: toml::Table) -> Self {
        Extractor {
            table: table,
            path: String::new(),
        }
    }
    fn full_name(&self, name: &str) -> String {
        if self.path.is_empty() {
            name.to_owned()
        } else {
            format!("{}.{}", self.path, name)
        }
    }
    fn conv<T: TryFromValue>(&self, value: &toml::Value, name: &str) -> Result<T, ExtractError> {
        match T::from_value(value) {
            Ok(value) => Ok(value),
            Err(type_str) => {
                Err(ExtractError::TypeMismatch {
                    name: self.full_name(name),
                    expected: T::type_str(),
                    got: type_str,
                })
            }
        }
    }
    pub fn require<T: TryFromValue>(&self, name: &str) -> Result<T, ExtractError> {
        let value = try!(self.require_value(name));
        self.conv(value, name)
    }
    pub fn optional<T: TryFromValue>(&self, name: &str) -> Option<Result<T, ExtractError>> {
        self.table.get(name).map(|v| self.conv(v, name))
    }
    fn require_value(&self, name: &str) -> Result<&toml::Value, ExtractError> {
        self.table.get(name).ok_or(ExtractError::Missing { name: self.full_name(name) })
    }
    pub fn require_table(&self, name: &str) -> Result<Self, ExtractError> {
        let value = try!(self.require_value(name));

        match *value {
            toml::Value::Table(ref t) => {
                Ok(Extractor {
                    table: t.clone(),
                    path: self.full_name(name),
                })
            }
            _ => {
                Err(ExtractError::TypeMismatch {
                    name: self.full_name(name),
                    expected: "table",
                    got: value.type_str(),
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum ExtractError {
    Missing { name: String },
    TypeMismatch {
        name: String,
        expected: &'static str,
        got: &'static str,
    },
}

#[test]
fn test_extractor() {
    use self::ExtractError::TypeMismatch;

    let toml = r#"
    skeleton = "skeleton.noten"
    index = "homepage"
    table = 10

    [directories]
    input = "in"
    output = "out"
    generators = "gen"

    [constants]
    foo = 77
    bar = "I am a constant"
    "#;
    let mut parser = toml::Parser::new(toml);
    let table = parser.parse().unwrap();

    let extractor = Extractor::new(table);
    let skel: String = extractor.require("skeleton").unwrap();
    assert_eq!(skel, "skeleton.noten");
    assert_eq!(extractor.require::<String>("index").unwrap(), "homepage");
    let dirs_ex = extractor.require_table("directories").unwrap();
    assert_eq!(dirs_ex.require::<String>("input").unwrap(), "in");
    let constants_ex = extractor.require_table("constants").unwrap();
    assert_eq!(constants_ex.require::<i64>("foo").unwrap(), 77);
    match constants_ex.require::<i64>("bar") {
        Err(TypeMismatch { ref name, expected: "integer", got: "string" }) if name ==
                                                                              "constants.bar" => {}
        etc => panic!("Didn't get error we expected, got: {:?}", etc),
    }
    println!("{:?}", extractor.table.get("skeleton"));
    match extractor.optional::<toml::Table>("table") {
        Some(Err(_)) => {}
        etc => panic!("Didn't get Some(Err), got: {:?}", etc),
    }
}
