extern crate clap;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::collections::HashMap;
use std::io::{self, Write};

use clap::{Arg, App};
use futures::{Future, Stream};
use hyper::{Client, Method, Request, Uri};
use tokio_core::reactor::Core;

fn main() -> Result<(), impl ::std::error::Error> {
    let matches = App::new("REQuesteR")
        .version("1.0.0")
        .author("Livio Ribeiro")
        .about("Perform http requests")
        .arg(Arg::with_name("method_get")
            .long("get")
            .conflicts_with_all(&["method_post", "method_put", "method_delete"]))
        .arg(Arg::with_name("method_post")
            .long("post")
            .conflicts_with_all(&["method_get", "method_put", "method_delete"]))
        .arg(Arg::with_name("method_put")
            .long("put")
            .conflicts_with_all(&["method_get", "method_post", "method_delete"]))
        .arg(Arg::with_name("method_delete")
            .long("delete")
            .conflicts_with_all(&["method_get", "method_post", "method_put"]))
        .arg(Arg::with_name("url")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("body")
            .short("B")
            .long("body")
            .takes_value(true)
            .multiple(true)
            .number_of_values(2))
        .arg(Arg::with_name("query")
            .short("Q")
            .long("query")
            .takes_value(true)
            .multiple(true)
            .number_of_values(2))
        .arg(Arg::with_name("header")
            .short("H")
            .long("header")
            .takes_value(true)
            .multiple(true)
            .number_of_values(2))
        .get_matches();

    let body: HashMap<String, String> = if let Some(body) = matches.values_of("body") {
        let (keys, values): (Vec<(usize, String)>, Vec<(usize, String)>) = body.into_iter()
            .map(|x| x.to_owned())
            .enumerate()
            .partition(|(i, _)| i % 2 == 0);
        keys.into_iter().map(|(_, x)| x).zip(values.into_iter().map(|(_, x)| x)).collect()
    } else {
        HashMap::new()
    };

    let method = if matches.is_present("method_get") {
        Method::Get
    } else if matches.is_present("method_post") {
        Method::Post
    } else if matches.is_present("method_put") {
        Method::Put
    } else if matches.is_present("method_delete") {
        Method::Delete
    } else {
        Method::Get
    };

    let url = matches.value_of("url").unwrap();

    let mut core = Core::new()?;
    let client = Client::new(&core.handle());

    let uri: Uri = url.parse()?;
    let req = match method {
        Method::Get | Method::Delete => {
            client.request(Request::new(method, uri))
        },
        Method::Post | Method::Put => {
            let mut req = Request::new(method, uri);
            req.set_body(serde_json::to_string(&body).unwrap());
            client.request(req)
        },
        _ => unreachable!()
    };

    let work = req.and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map(|_| ())
                .map_err(From::from)
        })
    });

    core.run(work)
}
