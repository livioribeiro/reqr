extern crate clap;
extern crate url;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
// #[macro_use] extern crate serde_derive;

mod parsers;

use std::io::{self, Write};

use clap::{Arg, App, ArgGroup, AppSettings};
use hyper::{Client, Request, Uri, Body};
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
        .group(ArgGroup::with_name("method")
            .args(&METHODS))
        .arg(Arg::with_name("GET")
            .long("get")
            .conflicts_with("body"))
        .arg(Arg::with_name("POST")
            .long("post"))
        .arg(Arg::with_name("PUT")
            .long("put"))
        .arg(Arg::with_name("DELETE")
            .long("delete")
            .conflicts_with("body"))
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

    if let Some(query) = matches.values_of("query") {
        let query_string = parsers::query_string(query);
        url = format!("{}?{}", url, query_string);
    }

    let uri: Uri = url.parse().unwrap();

    let mut request_builder = if matches.is_present("GET") {
        Request::get(uri)
    } else if matches.is_present("POST") {
        Request::post(uri)
    } else if matches.is_present("PUT") {
        Request::put(uri)
    } else if matches.is_present("DELETE") {
        Request::delete(uri)
    } else {
        if matches.is_present("body") || matches.is_present("json") || matches.is_present("form") {
            Request::post(uri)
        } else {
            Request::get(uri)
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
