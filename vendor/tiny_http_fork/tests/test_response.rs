
extern crate tiny_http_fork;

use std::borrow::Cow;

use tiny_http_fork::{Header, Response};

#[test]
fn test_response_from()
{   
    let mut resp = Response::empty(200);

    let headers: Vec<(Cow<'static, str>, Cow<'static, str>)> = 
        vec![
            (Cow::Borrowed("HeaderHeader"), Cow::Owned("data".to_string())),
            (Cow::Borrowed("HeaderHeaderHeader"), Cow::Borrowed("data32323"))
        ];

    resp = resp.try_with_headers(headers).unwrap();

    assert_eq!(resp.headers().contains("HeaderHeader"), true);
    assert_eq!(resp.headers().contains("HeaderHeaderHeader"), true);
  

    let headers: Vec<Header> = vec![Header::try_from("Header-Header: datagram").unwrap()];

    resp = resp.with_headers(headers).unwrap();

    assert_eq!(resp.headers().contains("HeaderHeader"), true);
    assert_eq!(resp.headers().contains("HeaderHeaderHeader"), true);
    assert_eq!(resp.headers().contains("Header-Header"), true);
}