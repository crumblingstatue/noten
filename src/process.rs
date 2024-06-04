use {
    crate::{
        config::Config, skeleton::Skeleton, substitution::substitute, template_deps::TemplateDeps,
    },
    hoedown::{self, Html, Markdown, Render},
    lazy_static::lazy_static,
    log::debug,
    serde_derive::Deserialize,
    std::{error::Error, path::Path},
};

#[derive(Default, Deserialize)]
struct Attributes {
    title: Option<String>,
    description: Option<String>,
    constants: Option<toml::value::Table>,
}

/// Reads the optional attribute section at the beginning of the template.
///
/// Returns `Attributes`, and the end position of the attribute section.
fn read_attributes(input: &str) -> (Attributes, usize) {
    let first_char = input.chars().next().expect("Couldn't get first character");
    if first_char != '{' {
        return (Default::default(), 0);
    }
    let closing_brace_pos = input.find('}').expect("Expected closing }");
    let end = closing_brace_pos + 1;
    let attribs = toml::from_str(&input[1..closing_brace_pos]).unwrap();
    (attribs, end)
}

fn find_title(input: &str) -> Result<&str, Box<dyn Error>> {
    use regex::Regex;

    lazy_static! {
        static ref MD: Regex = Regex::new("#{1, 9}(.*)").unwrap();
        static ref HTML: Regex = Regex::new("<h[0-9]>(.*)</h[0-9]>").unwrap();
    }
    // The first non-empty line will be tried as the title.
    let line = match input.lines().find(|l| !l.is_empty()) {
        Some(line) => line,
        None => return Err("There are only empty lines in the document".into()),
    };
    // Try a markdown header first
    match MD.captures(line) {
        Some(caps) => Ok(caps.get(1).unwrap().as_str().trim()),
        None => {
            // Try the HTML header
            match HTML.captures(line) {
                Some(caps) => Ok(caps.get(1).unwrap().as_str().trim()),
                None => Err(format!("\"{}\" is not a valid header", line).into()),
            }
        }
    }
}

#[test]
fn test_find_title() {
    ::env_logger::init();
    assert_eq!(
        find_title("## Tales of Something\n").unwrap(),
        "Tales of Something"
    );
    assert_eq!(find_title("# Masszázs\n").unwrap(), "Masszázs");
    assert_eq!(find_title("<h2>Elérhetőség</h2>\n").unwrap(), "Elérhetőség");
    assert_eq!(
        find_title("<h2> Asszisztok betegségekre és sérülésekre </h2>").unwrap(),
        "Asszisztok betegségekre és sérülésekre"
    );
    assert_eq!(find_title("<h2>Title</h2>\n# Junk\n").unwrap(), "Title");
}

pub struct ProcessingContext<'a> {
    pub template_path: &'a Path,
    pub template_deps: &'a mut TemplateDeps,
    pub config: &'a Config,
}

/// Process a template
pub fn process(
    input: &str,
    context: &mut ProcessingContext,
    skeleton: &Skeleton,
) -> Result<String, Box<dyn Error>> {
    context.template_deps.clear_deps(context.template_path);
    let mut output = String::new();
    let (attribs, mut from) = read_attributes(input);
    let title = match attribs.title {
        Some(title) => title,
        None => find_title(&input[from..])?.to_owned(),
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
                match substitute(substitution, context, attribs.constants.as_ref()) {
                    Ok(text) => output.push_str(&text),
                    Err(e) => return Err(format!("Error handling substitution: {}", e).into()),
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
    let render_result = html.render(&doc);
    let output = render_result.to_str().expect("markdown=>html failed");
    skeleton.out(&title, output, attribs.description.as_ref().map(|s| &s[..]))
}
