#![allow(dead_code)]

use std::str;


#[derive(Debug)]
pub struct Response {
    pub size: String,
    pub source: String,
    pub mac_address: String,
    pub firmware: String,
    pub sequence_number: String,
    pub reserved_1: String,
    pub message_type: String,
    pub reserved_2: String,
    pub payload: Payload,
}

pub fn parse_response(resp_msg: ResponseMessage) -> Response {
    let mut resp = parse_header(&resp_msg);

    let payload = match resp.message_type.as_str() {
        "3" => parse_payload_3(&resp_msg), 
        "107" => parse_payload_107(&resp_msg),
        _ => Payload::None(()),
    };

    resp.payload = payload;

    resp
}

fn parse_header(resp: &ResponseMessage) -> Response {
    Response {
        size: ResponseMessage::size(&resp),
        source: ResponseMessage::source(&resp),
        mac_address: ResponseMessage::mac_address(&resp),
        firmware: ResponseMessage::firmware(&resp),

        // TODO: packed byte
        sequence_number: ResponseMessage::sequence_number(&resp),

        // Message segment: protocol header
        reserved_1: ResponseMessage::reserved_1(&resp), // timestamp?
        message_type: ResponseMessage::message_type(&resp),
        reserved_2: ResponseMessage::reserved_2(&resp),
        payload: Payload::None(()),
    }
}

#[derive(Debug)]
pub enum Payload {
    None(()),
    StateService(StateServicePayload),
    State(StatePayload),
}

#[derive(Debug)]
pub struct StateServicePayload {
    service: String,
    port: String,
    unknown: String,
}

#[derive(Debug)]
pub struct StatePayload {
    body: String,
    hue: String,
    saturation: String,
    brightness: String,
    kelvin: String,
}

fn parse_payload_3(resp: &ResponseMessage) -> Payload {
    Payload::StateService(StateServicePayload {
        service: ResponseMessage::service(&resp),
        port: ResponseMessage::port(&resp),
        unknown: ResponseMessage::unknown(&resp),
    })
}

fn parse_payload_107(resp: &ResponseMessage) -> Payload {
    Payload::State(StatePayload {
        body: ResponseMessage::body(&resp),
        hue: ResponseMessage::hue(&resp),
        saturation: ResponseMessage::saturation(&resp),
        brightness: ResponseMessage::brightness(&resp),
        kelvin: ResponseMessage::kelvin(&resp),
    })
}

pub struct ResponseMessage(pub Vec<u8>);

impl ResponseMessage {
    fn size(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 0, 2);
        b.reverse();
        as_base10(b)
    }

    fn source(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 4, 4);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn mac_address(resp: &ResponseMessage) -> String {
        as_hex(extract(&resp, 8, 8))
    }

    fn firmware(resp: &ResponseMessage) -> String {
        as_ascii(extract(&resp, 16, 6))
    }

    fn sequence_number(resp: &ResponseMessage) -> String {
        as_base10(extract(&resp, 23, 1))
    }

    fn reserved_1(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 24, 8);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn message_type(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 32, 2);
        b.reverse();
        as_base10(b)
    }

    fn reserved_2(resp: &ResponseMessage) -> String {
        let b = extract(&resp, 34, 2);
        as_base10(b)  // TODO: may not be base10, but undocumented.
    }

    fn service(resp: &ResponseMessage) -> String {
        as_base10(extract(&resp, 36, 1))
    }

    fn port(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 37, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn unknown(resp: &ResponseMessage) -> String {
        let end = resp.0.len() - 39;
        let b = extract(&resp, 39, end);
        // as_base10(b)  // TODO: may not be base10, but undocumented.
        as_hex(b)
    }

    fn body(resp: &ResponseMessage) -> String {
        let end = resp.0.len() - 36;
        as_hex(extract(&resp, 36, end))
    }

    fn hue(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 36, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn saturation(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 38, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn brightness(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 40, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn kelvin(resp: &ResponseMessage) -> String {
        let mut b = extract(&resp, 42, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }
}

fn extract(resp: &ResponseMessage, start: usize, len: usize) -> Vec<u8> {
    let mut sub = vec![0u8; len];
    sub[..len].clone_from_slice(&resp.0[start..start + len]);
    sub
}

fn as_base10(v: Vec<u8>) -> String {
    let mut s = "".to_string();
    for b in v {
        s.push_str(format!("{}", b).as_str());
    }
    let n = s.parse::<u16>().unwrap();
    n.to_string()
}

fn as_ascii(arr: Vec<u8>) -> String {
    str::from_utf8(&arr).unwrap().to_string()
}

fn as_boolean(v: Vec<u8>) -> String {
    let mut s = "".to_string();
    for b in v {
        s.push_str(format!("{:08b}", b).as_str());
    }
    s
}

fn as_hex(arr: Vec<u8>) -> String {
    let mut s: Vec<String> = vec![];
    for b in arr {
        s.push(format!("{:02X}", b));
    }
    s.join(":")
}

fn bitstr_to_u32(bits: &str) -> u32 {
    bits.as_bytes().iter().fold(0, |acc, b| (acc << 1) + if *b == 48 { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::{ResponseMessage, extract, as_base10, as_ascii, as_boolean, as_hex, bitstr_to_u32};

    #[test]
    fn test_extract() {
        let resp = ResponseMessage(vec![41, 42, 43, 44, 45, 46, 47, 48, 49]);
        assert_eq!(extract(&resp, 2, 3), vec![43, 44, 45]);
    }

    #[test]
    fn test_as_base10() {
        assert_eq!(as_base10(vec![00, 41]), "41");
    }

    #[test]
    fn test_as_boolean() {
        assert_eq!(as_boolean(vec![221, 124]), "1101110101111100");
    }

    #[test]
    fn test_bitstr_to_u32() {
        assert_eq!(bitstr_to_u32("1101110101111100"), 56700);
    }

    #[test]
    fn test_as_ascii() {
        assert_eq!(as_ascii(vec![76, 73, 70, 88, 86, 50]), "LIFXV2");
    }

    #[test]
    fn test_from_hex() {
        assert_eq!(as_hex(vec![209, 114, 214, 20, 224, 14, 0, 0]),
                   "D1:72:D6:14:E0:0E:00:00");
    }
}
