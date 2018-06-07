use std::collections::HashMap;
use hyper::header::{HeaderName, HeaderValue};
use serde_json;
use url::form_urlencoded::Serializer as UrlEncodedSerializer;

pub enum BodyFormat {
    JSON, FORM
}

pub fn query_string<'a>(query_list: impl Iterator<Item = &'a str>) -> String {
    let (keys, values): (Vec<(usize, &str)>, Vec<(usize, &str)>) = query_list.into_iter()
        .enumerate()
        .partition(|(i, _)| i % 2 == 0);

    let data = keys.into_iter().map(|(_, x)| x)
        .zip(values.into_iter().map(|(_, x)| x));

    UrlEncodedSerializer::new(String::new()).extend_pairs(data).finish()
}

pub fn headers<'a>(header_list: impl Iterator<Item = &'a str>) -> Vec<(HeaderName, HeaderValue)> {
    let (keys, values): (Vec<(usize, String)>, Vec<(usize, String)>) = header_list.into_iter()
        .map(|x| x.to_owned())
        .enumerate()
        .partition(|(i, _)| i % 2 == 0);

    keys.into_iter().map(|(_, x)| x)
        .zip(values.into_iter().map(|(_, x)| x))
        .map(|(key, value)| {
            let hkey = HeaderName::from_bytes(key.as_bytes()).unwrap();
            let hvalue = HeaderValue::from_bytes(value.as_bytes()).unwrap();
            (hkey, hvalue)
        })
        .collect()
}

pub fn body<'a>(attr_list: impl Iterator<Item = &'a str>, format: BodyFormat) -> String {
    let (keys, values): (Vec<(usize, String)>, Vec<(usize, String)>) = attr_list.into_iter()
            .map(|x| x.to_owned())
            .enumerate()
            .partition(|(i, _)| i % 2 == 0);

    let data: Vec<_> = keys.into_iter().map(|(_, x)| x)
        .zip(values.into_iter().map(|(_, x)| x))
        .collect();

    match format {
        BodyFormat::JSON => serde_json::ser::to_string(&data.into_iter().collect::<HashMap<String, String>>()).unwrap(),
        BodyFormat::FORM => UrlEncodedSerializer::new(String::new()).extend_pairs(data).finish()
    }
}