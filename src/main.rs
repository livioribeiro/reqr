#[macro_use] extern crate clap;
extern crate url;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate syntect;

mod parsers;

// use std::io::{self, Write};

use clap::{Arg, App, AppSettings};
use hyper::{Client, Request, Body};
use hyper::rt::{self, lazy, Future, Stream};
use hyper_tls::HttpsConnector;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::as_24_bit_terminal_escaped;

use parsers::BodyFormat;

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
            .short("o"))
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
        .arg(Arg::with_name("header")
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

    let request = request_builder.body(request_body).unwrap();

    rt::run(lazy(|| {
        let https = HttpsConnector::new(4).expect("TLS initialization failed");
        let client = Client::builder().build::<_, hyper::Body>(https);

        client.request(request).and_then(|res| {
            println!("Response: {}", res.status());
            res
                .into_body()
                .concat2()
                .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
                .and_then(|s| {
                    let ps = SyntaxSet::load_defaults_nonewlines();
                    let ts = ThemeSet::load_defaults();
                    let syntax = ps.find_syntax_by_extension("html").unwrap();
                    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

                    for line in s.lines() {
                        let ranges: Vec<(Style, &str)> = h.highlight(line);
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                        println!("{}", escaped);
                    }

                    Ok(())
                })
            })
            .map_err(|err| {
                println!("Error: {}", err);
            })
    }));
}
