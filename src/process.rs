use config::Config;
use toml;
use substitution::substitute;
use hoedown::{self, Html, Markdown, Render};
use std::error::Error;
use std::path::Path;
use template_deps::TemplateDeps;

#[derive(Default)]
struct Attributes {
    title: Option<String>,
}

/// Reads the optional attribute section at the beginning of the template.
///
/// Reurns `Attributes`, and the end position of the attribute section.
fn read_attributes(input: &str) -> Result<(Attributes, usize), Box<Error>> {
    let first_char = input.chars().next().expect("Couldn't get first character");
    if first_char != '{' {
        return Ok((Default::default(), 0));
    }
    let closing_brace_pos = input.find('}').expect("Expected closing }");
    let attribs = {
        let attribs = &input[1..closing_brace_pos];
        let mut parser = toml::Parser::new(attribs);
        match parser.parse() {
            Some(toml) => toml,
            None => {
                let toml_error = ::util::toml::parser_error_to_string(&parser);
                let msg = format!("Failed to parse attribute TOML:\n{}", toml_error);
                return Err(msg.into());
            }
        }
    };
    let title = attribs.get("title").and_then(|v| v.as_str()).map(|s| s.to_owned());
    let end = closing_brace_pos + 1;
    let attribs = Attributes { title: title };
    Ok((attribs, end))
}

fn text_of_first_header_html(input: &str) -> Option<&str> {
    use regex::Regex;
    debug!("Getting title from html header \"{}\"", input);
    let re = Regex::new(r"<h[0-9]>(.*)</h[0-9]>").unwrap();
    let caps = match re.captures(input) {
        Some(caps) => caps,
        None => return None,
    };
    debug!("Got captures");
    caps.at(1)
}

fn text_of_first_header(input: &str) -> Option<&str> {
    let first_hash = match input.find('#') {
        Some(pos) => pos,
        None => return text_of_first_header_html(input),
    };
    debug!("Found first_hash: {}", first_hash);
    let first_space = match input[first_hash..].find(' ') {
        Some(pos) => first_hash + pos,
        None => return None,
    };
    debug!("Found first_space: {}", first_space);
    let first_newline = match input[first_space..].find('\n') {
        Some(pos) => first_space + pos,
        None => return None,
    };
    debug!("Found first_newine: {}", first_newline);
    Some(&input[first_space + 1..first_newline])
}

#[test]
fn test_text_of_first_header() {
    ::env_logger::init().unwrap();
    assert_eq!(text_of_first_header("## Tales of Something\n"),
               Some("Tales of Something"));
    assert_eq!(text_of_first_header("# Masszázs\n"), Some("Masszázs"));
    assert_eq!(text_of_first_header("<h2>Elérhetőség</h2>\n"),
               Some("Elérhetőség"));
}

pub struct ProcessingContext<'a> {
    pub template_path: &'a Path,
    pub template_deps: &'a mut TemplateDeps,
}

/// Process a template
pub fn process(input: String,
               config: &Config,
               context: &mut ProcessingContext)
               -> Result<String, Box<Error>> {
    context.template_deps.clear_deps(context.template_path);
    let mut output = String::new();
    let (attribs, mut from) = try!(read_attributes(&input));
    let title = match attribs.title {
        Some(title) => title,
        None => try!(text_of_first_header(&input).ok_or("Couldn't get title")).to_owned(),
    };
    loop {
        debug!("Attempting to find next {{{{ or EOF @ {}", from);
        // Just copy the content as-is until the next {{ or EOF
        match input[from..].find("{{") {
            Some(pos) => {
                debug!("Found {{{{ @ {}", pos);
                output.push_str(&input[from..from + pos]);
                let closing_pos = input[from + pos..].find("}}").expect("Expected closing }}");
                let substitution = &input[from + pos + 2..from + pos + closing_pos];
                match substitute(substitution, config, context) {
                    Ok(text) => output.push_str(&text),
                    Err(e) => panic!("Error handling substitution: {}", e),
                }
                debug!("Substitution: \"{}\"", substitution);
                from = from + pos + closing_pos + 2;
            }
            None => {
                output.push_str(&input[from..]);
                break;
            }
        }
    }
    let doc = Markdown::new(&output).extensions(hoedown::TABLES);
    let mut html = Html::new(hoedown::renderer::html::Flags::empty(), 0);
    Ok(format!("<?php
function title()
{{
    return \"{title}\";
}}

function content()
{{
?>
{output}
<?php
}}
?>
",
               title = title,
               output = html.render(&doc).to_str().expect("markdown=>html failed")))
}
