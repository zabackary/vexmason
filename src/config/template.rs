use std::{borrow::Cow, collections::HashMap};

use chrono::{Datelike, Local, Timelike};
use lazy_static::lazy_static;
use regex::Regex;

use super::model::ConfigDefineType;

lazy_static! {
    static ref TEMPLATE_REGEX: Regex =
        Regex::new(r"(?P<match>\{\{\s*(?P<content>.*?)\s*\}\})").unwrap();
}

pub fn evaluate_template<'a>(
    template: &'a str,
    computer_name: &'a str,
    language: &'a str,
    minify: bool,
    resolved_defines: &'a HashMap<String, ConfigDefineType>,
) -> Cow<'a, str> {
    TEMPLATE_REGEX.replace_all(template, |captures: &regex::Captures<'_>| {
        match &captures["content"] {
            "computer-name" => computer_name.to_string(),

            "language" => language.to_string(),
            "language::short" => match language {
                "python" => "py",
                "cpp" => "cpp",
                _ => "?",
            }
            .to_string(),

            "minify" => minify.to_string(),
            "minify::short" => if minify { "y" } else { "n" }.to_string(),

            "time" => Local::now().format("%a %d %b %Y, %I:%M%p").to_string(),
            "time::iso8601" => Local::now().format("%+").to_string(),
            "time/year" => Local::now().year().to_string(),
            "time/month" => Local::now().month().to_string(),
            "time/day" => Local::now().day().to_string(),
            "time/hour" => Local::now().hour().to_string(),
            "time/minute" => Local::now().minute().to_string(),

            "defines::list" => {
                let mut define_str = String::new();
                for (k, v) in resolved_defines {
                    define_str.push_str(k);
                    define_str.push_str("=");
                    define_str.push_str(&Into::<String>::into(v.clone()));
                    define_str.push_str(", ");
                }
                define_str
            }
            "defines::count" => resolved_defines.len().to_string(),
            content => {
                if let Some(define_name) = content.strip_prefix("defines/") {
                    resolved_defines.get(define_name).map_or_else(
                        || captures["match"].to_string(),
                        |a| Into::<String>::into(a.clone()),
                    )
                } else {
                    captures["match"].to_string()
                }
            }
        }
    })
}
