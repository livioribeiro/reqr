#[macro_use] extern crate clap;
extern crate url;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate syntect;

mod parsers;

use std::fs::File;
use std::io::Write;

use clap::{Arg, App, AppSettings};
use hyper::{Client, Request, Body};
use hyper::rt::{self, lazy, Future, Stream};
use hyper_tls::HttpsConnector;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::as_24_bit_terminal_escaped;

use parsers::BodyFormat;

const CONTENT_TYPE_MAP: &[(&str, &str)] = &[
    ("json", "json"),
    ("xml", "xml"),
    ("javascript", "js"),
    ("html", "html"),
    ("css", "css"),
];

fn main() {
    let matches = App::new("REQuesteR")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("method")
            .long("method")
            .short("m")
            .case_insensitive(true)
            .possible_values(&["GET", "POST", "PUT", "DELETE"])
            .default_value_if("body", None, "POST")
            .default_value_if("form", None, "POST")
            .default_value_if("json", None, "POST")
            .default_value("GET"))
        .arg(Arg::with_name("url")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .takes_value(true))
        .arg(Arg::with_name("json")
            .long("json")
            .conflicts_with("form"))
        .arg(Arg::with_name("form")
            .long("form"))
        .arg(Arg::with_name("body")
            .short("B")
            .long("body")
            .takes_value(true)
            .multiple(true)
            .number_of_values(2)
            .value_names(&["name", "value"]))
        .arg(Arg::with_name("query")
            .short("Q")
            .long("query")
            .takes_value(true)
            .multiple(true)
            .number_of_values(2)
            .value_names(&["name", "value"]))
        .arg(Arg::with_name("headers")
            .short("H")
            .long("header")
            .takes_value(true)
            .multiple(true)
            .number_of_values(2)
            .value_names(&["name", "value"]))
        .get_matches();

    let url = matches.value_of("url").unwrap().to_owned();

    let maybe_query = matches.values_of("query");
    let uri = parsers::uri(url, maybe_query).unwrap();

    let mut request_builder = match matches.value_of("method").unwrap_or("GET") {
        "GET" => Request::get(uri),
        "POST" => Request::post(uri),
        "PUT" => Request::put(uri),
        "DELETE" => Request::delete(uri),
        _ => unreachable!(),
    };

    if let Some(headers) = matches.values_of("headers") {
        for (key, value) in parsers::headers(headers) {
            request_builder.header(key, value);
        }
    }

    let request_body: Body = if let Some(body) = matches.values_of("body") {
        let format = if matches.is_present("form") {
            BodyFormat::FORM
        } else {
            BodyFormat::JSON
        };
        parsers::body(body, format).into()
    } else {
        Body::empty()
    };

    request_builder.header("user-agent", "reqr/0.1");

    let request = request_builder.body(request_body).unwrap();

    let maybe_file = matches.value_of("output").map(ToOwned::to_owned);

    rt::run(lazy(move || {
        let https = HttpsConnector::new(4).expect("TLS initialization failed");
        let client = Client::builder().build::<_, hyper::Body>(https);

        client.request(request).and_then(move |res| {

            let content_type = res.headers()
                .get("content-type")
                .map(|x| x.to_str().unwrap().to_owned());

            let syntax_highlight = content_type.as_ref().and_then(|content_type| {
                CONTENT_TYPE_MAP.iter()
                    .skip_while(|(ct, _)| !content_type.contains(ct))
                    .map(|(_, syntax)| *syntax)
                    .next()
            });

            println!("Response: {}", res.status());

            res
                .into_body()
                .concat2()
                .and_then(move |chunks| {
                    if let Some(path) = maybe_file {
                        let mut file = File::create(path).unwrap();
                        file.write_all(&chunks).unwrap();
                    } else {
                        let s = if syntax_highlight == Some("json") {
                            let parsed: serde_json::Value = serde_json::from_slice(&chunks).unwrap();
                            serde_json::to_string_pretty(&parsed).unwrap()
                        } else {
                            String::from_utf8_lossy(&chunks).into_owned()
                        };

                        if let Some(syntax_highlight) = syntax_highlight {
                            let ss = SyntaxSet::load_defaults_nonewlines();
                            let ts = ThemeSet::load_defaults();
                            let syntax = ss.find_syntax_by_extension(syntax_highlight).unwrap();
                            let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

                            for line in s.lines() {
                                let ranges: Vec<(Style, &str)> = h.highlight(line);
                                let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                                println!("{}", escaped);
                            }
                        } else {
                            println!("{}", s);
                        }
                    }

                    Ok(())
                })
            })
            .map_err(|err| {
                println!("Error: {}", err);
            })
    }));
}
