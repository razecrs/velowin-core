use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::multispace0,
    sequence::{delimited, tuple},
    IResult, Parser,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Config {
    pub general: HashMap<String, String>,
    pub decoration: HashMap<String, String>,
    pub animations: HashMap<String, String>,
}

fn parse_section_name(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphabetic())(input)
}

fn parse_section_body(input: &str) -> IResult<&str, &str> {
    delimited(
        (multispace0, tag("{"), multispace0),
        take_until("}"),
        tag("}"),
    ).parse(input)
}

pub fn parse_config(content: &str) -> Config {
    let mut config = Config::default();
    let mut input = content;

    while !input.is_empty() {
        let result: IResult<&str, (&str, &str)> = (
            multispace0,
            parse_section_name,
            parse_section_body,
        ).map(|(_, name, body)| (name, body))
        .parse(input);

        match result {
            Ok((next_input, (name, body))) => {
                let map = match name {
                    "general" => &mut config.general,
                    "decoration" => &mut config.decoration,
                    "animations" => &mut config.animations,
                    _ => {
                        input = next_input;
                        continue;
                    }
                };

                for line in body.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    if let Some((key, value)) = line.split_once('=') {
                        map.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
                input = next_input;
            }
            Err(_) => break,
        }
    }

    config
}
