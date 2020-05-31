use std::error::Error;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug)]
enum Segment {
    Content,
    Description,
    IfDesc(Vec<Segment>),
    Text(String),
    Title,
}

pub struct Skeleton {
    segments: Vec<Segment>,
}

#[derive(Debug)]
enum Token<'a> {
    Content,
    Description,
    EndIfDesc,
    IfDesc,
    LiteralText(&'a str),
    Title,
}

fn lex(text: &str) -> Result<Vec<Token>, Box<Error>> {
    let mut tokens = Vec::new();
    let mut rest = text;
    while let Some(begin) = rest.find("%(") {
        tokens.push(Token::LiteralText(&rest[..begin]));
        rest = &rest[begin + 2..];
        let end = match rest.find(')') {
            Some(pos) => pos,
            None => return Err("`%(` without matching `)`".into()),
        };
        let keyword = &rest[..end];
        let token = match keyword {
            "content" => Token::Content,
            "description" => Token::Description,
            "endifdesc" => Token::EndIfDesc,
            "ifdesc" => Token::IfDesc,
            "title" => Token::Title,
            _ => return Err(format!("Unknown keyword `{}`", keyword).into()),
        };
        tokens.push(token);
        rest = &rest[end + 1..];
    }
    tokens.push(Token::LiteralText(rest));
    Ok(tokens)
}

fn parse(tokens: &[Token]) -> Result<Vec<Segment>, Box<Error>> {
    enum State {
        TopLevel,
        IfDesc,
    }

    let mut segments: Vec<Segment> = Vec::new();
    let mut if_segs = Vec::new();
    let mut iter = tokens.iter();
    let mut state = State::TopLevel;
    macro_rules! which {
        () => {
            match state {
                State::TopLevel => &mut segments,
                State::IfDesc => &mut if_segs,
            }
        };
    }

    loop {
        let tok = iter.next();
        match tok {
            Some(&Token::Content) => which!().push(Segment::Content),
            Some(&Token::Description) => which!().push(Segment::Description),
            Some(&Token::EndIfDesc) => match state {
                State::TopLevel => return Err("endifdesc without preceding ifdesc".into()),
                State::IfDesc => {
                    use std::mem;
                    let if_segs = mem::replace(&mut if_segs, Vec::new());
                    segments.push(Segment::IfDesc(if_segs));
                    state = State::TopLevel;
                }
            },
            Some(&Token::IfDesc) => match state {
                State::TopLevel => state = State::IfDesc,
                State::IfDesc => return Err("Nested ifdescs are not supported".into()),
            },
            Some(&Token::LiteralText(text)) => which!().push(Segment::Text(text.to_owned())),
            Some(&Token::Title) => which!().push(Segment::Title),
            None => return Ok(segments),
        }
    }
}

impl Skeleton {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<(Self, SystemTime), Box<Error>> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let tokens = lex(&s)?;
        debug!("Got tokens: {:#?}", tokens);
        let segments = parse(&tokens)?;
        debug!("Got segments: {:#?}", segments);
        Ok((
            Skeleton { segments: segments },
            f.metadata().unwrap().modified().unwrap(),
        ))
    }
    pub fn out(
        &self,
        title: &str,
        content: &str,
        description: Option<&str>,
    ) -> Result<String, Box<Error>> {
        out_segs(&self.segments, title, content, description)
    }
}

fn out_segs(
    segments: &[Segment],
    title: &str,
    content: &str,
    description: Option<&str>,
) -> Result<String, Box<Error>> {
    let mut out = String::new();
    for seg in segments {
        let string;
        let s = match *seg {
            Segment::Content => content,
            Segment::Description => match description {
                Some(desc) => desc,
                None => {
                    return Err("Tried to get description when it didn't exist. \
                         Try putting it in an ifdesc block."
                        .into())
                }
            },
            Segment::IfDesc(ref segs) => match description {
                Some(desc) => {
                    string = out_segs(segs, title, content, Some(desc))?;
                    &string
                }
                None => "",
            },
            Segment::Text(ref text) => text,
            Segment::Title => title,
        };
        out.push_str(s);
    }
    Ok(out)
}
