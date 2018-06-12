use std::collections::HashMap;
use hyper::Uri;
use hyper::header::{HeaderName, HeaderValue};
use serde_json;
use url::Url;
use url::form_urlencoded::Serializer as UrlEncodedSerializer;

pub enum BodyFormat {
    JSON, FORM
}

pub fn uri<'a, T: Iterator<Item = &'a str>>(mut url: String, query: Option<T>) -> Result<Uri, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("http://{}", url);
    }

    if let Some(query) = query {
        let (keys, values): (Vec<(usize, &str)>, Vec<(usize, &str)>) = query.into_iter()
            .enumerate()
            .partition(|(i, _)| i % 2 == 0);
        let query = keys.into_iter().map(|(_, x)| x)
            .zip(values.into_iter().map(|(_, x)| x));

        let parsed = Url::parse_with_params(&url, query).map_err(|x| format!("{}", x))?;
        Uri::from_shared(parsed.as_str().as_bytes().into()).map_err(|x| format!("{}", x))
    } else {
        url.parse().map_err(|x| format!("{}", x))
    }
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
    let (keys, values): (Vec<(usize, &str)>, Vec<(usize, &str)>) = attr_list.into_iter()
            .enumerate()
            .partition(|(i, _)| i % 2 == 0);

    let data = keys.into_iter().map(|(_, x)| x)
        .zip(values.into_iter().map(|(_, x)| x));

    match format {
        BodyFormat::JSON => serde_json::ser::to_string(&data.into_iter().collect::<HashMap<&str, &str>>()).unwrap(),
        BodyFormat::FORM => UrlEncodedSerializer::new(String::new()).extend_pairs(data).finish()
    }
}