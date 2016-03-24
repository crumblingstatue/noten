use config::Config;
use toml;
use substitution::substitute;
use hoedown::{self, Html, Markdown, Render};

quick_error! {
    #[derive(Debug)]
    pub enum Error {

    }
}

/// Process a template
pub fn process(input: String, config: &Config) -> Result<String, Error> {
    let first_char = input.chars().next().expect("Couldn't get first character");
    if first_char != '{' {
        panic!("First character must be {");
    }
    let closing_brace_pos = input.find('}').expect("Expected closing }");
    let attribs = {
        let attribs = &input[1..closing_brace_pos];
        let mut parser = toml::Parser::new(attribs);
        parser.parse().expect("Failed to parse attribs TOML")
    };
    let mut output = String::new();
    let mut from = closing_brace_pos + 1;
    let title = attribs.get("title")
                       .expect("Attribute \"title\" is required, but not found")
                       .as_str()
                       .expect("Attribute \"title\" must be string");
    loop {
        debug!("Attempting to find next {{{{ or EOF @ {}", from);
        // Just copy the content as-is until the next {{ or EOF
        match input[from..].find("{{") {
            Some(pos) => {
                debug!("Found {{{{ @ {}", pos);
                output.push_str(&input[from..from + pos]);
                let closing_pos = input[from + pos..].find("}}").expect("Expected closing }}");
                let substitution = &input[from + pos + 2..from + pos + closing_pos];
                match substitute(substitution, config) {
                    Ok(text) => output.push_str(&text),
                    Err(e) => panic!("Error handling substitution: {}", e),
                }
                debug!("Substitution: \"{}\"", substitution);
                from = from + pos + closing_pos;
            }
            None => {
                output.push_str(&input[from + 2..]);
                break;
            }
        }
    }
    let doc = Markdown::new(&output);
    let mut html = Html::new(hoedown::renderer::html::Flags::empty(), 0);
    Ok(format!("<?php
function title() {{
    return \"{title}\";
}}
function content() {{
?>{output}
<?php
}}
?>
",
               title = title,
               output = html.render(&doc).to_str().expect("markdown=>html failed")))
}
