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
