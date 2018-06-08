extern crate clap;
extern crate url;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;

mod parsers;

use std::io::{self, Write};

use clap::{Arg, App, AppSettings};
use hyper::{Client, Request, Body};
use hyper::rt::{self, lazy, Future, Stream};
use hyper_tls::HttpsConnector;

use parsers::BodyFormat;

const METHODS: [&str; 4] = ["GET", "POST", "PUT", "DELETE"];

fn main() { // -> Result<(), impl ::std::error::Error> {
    let matches = App::new("REQuesteR")
        .version("1.0.0")
        .author("Livio Ribeiro")
        .about("Perform http requests")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("method")
            .long("method")
            .possible_values(&METHODS))
        .arg(Arg::with_name("url")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("json")
            .long("json"))
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

    let mut url = matches.value_of("url").unwrap().to_owned();

    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("http://{}", url);
    }

    let maybe_query = matches.values_of("query");
    let uri = parsers::uri(&url, maybe_query).unwrap();

    let mut request_builder = match matches.value_of("method") {
        Some("GET") => Request::get(uri),
        Some("POST") => Request::post(uri),
        Some("PUT") => Request::put(uri),
        Some("DELETE") => Request::delete(uri),
        _ => {
            if matches.is_present("body") || matches.is_present("json") || matches.is_present("form") {
                Request::post(uri)
            } else {
                Request::get(uri)
            }
        }
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
                // Body is a stream, so as each chunk arrives...
                .for_each(|chunk| {
                    io::stdout()
                        .write_all(&chunk)
                        .map_err(|e| {
                            panic!("example expects stdout is open, error={}", e)
                        })
                })
            })
            .map_err(|err| {
                println!("Error: {}", err);
            })
    }));
}
