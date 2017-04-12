#![allow(dead_code)]

use std::str;

#[derive(Debug)]
pub struct Response {
    pub size: u16,
    pub source: u32,
    pub mac_address: String,
    pub firmware: String,
    pub sequence_number: u16,
    pub reserved_1: u32,
    pub message_type: u16,
    pub reserved_2: u16,
    pub payload: Payload,
}

#[derive(Debug)]
pub struct ResponseString {
    pub size: String,
    pub source: String,
    pub mac_address: String,
    pub firmware: String,
    pub sequence_number: String,
    pub reserved_1: String,
    pub message_type: String,
    pub reserved_2: String,
    pub payload: PayloadString,
}

pub fn parse_response(resp_msg: ResponseData) -> Response {
    let mut resp = parse_header(&resp_msg);

    let payload = match resp.message_type {
        3 => parse_payload_3(&resp_msg), 
        107 => parse_payload_107(&resp_msg),
        _ => Payload::None(()),
    };

    resp.payload = payload;

    resp
}
pub fn parse_response_string(resp_msg: ResponseMessage) -> ResponseString {
    let mut resp = parse_header_string(&resp_msg);

    let payload = match resp.message_type.as_str() {
        "3" => parse_payload_3_string(&resp_msg), 
        "107" => parse_payload_107_string(&resp_msg),
        _ => PayloadString::None(()),
    };

    resp.payload = payload;

    resp
}

fn parse_header(resp: &ResponseData) -> Response {
    Response {
        size: ResponseData::size(resp),
        source: ResponseData::source(resp),
        mac_address: ResponseData::mac_address(resp),
        firmware: ResponseData::firmware(resp),

        // TODO: packed byte
        sequence_number: ResponseData::sequence_number(&resp),

        // Message segment: protocol header
        reserved_1: ResponseData::reserved_1(&resp), // timestamp?
        message_type: ResponseData::message_type(&resp),
        reserved_2: ResponseData::reserved_2(&resp),
        payload: Payload::None(()),
    }
}
fn parse_header_string(resp: &ResponseMessage) -> ResponseString {
    ResponseString {
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
        payload: PayloadString::None(()),
    }
}

#[derive(Debug)]
pub enum Payload {
    None(()),
    StateService(StateServicePayload),
    State(StatePayload),
}

#[derive(Debug)]
pub enum PayloadString {
    None(()),
    StateService(StateServicePayloadString),
    State_String(StatePayload_String),
}

#[derive(Debug)]
pub struct StateServicePayload {
    pub service: u16, // String,
    pub port: u32, // String,
    pub unknown: String,
}
#[derive(Debug)]
pub struct StateServicePayloadString {
    pub service: String,
    pub port: String,
    pub unknown: String,
}

#[derive(Debug)]
pub struct StatePayload {
    pub body: String,
    pub hsbk: PayloadHSBK,
}

#[derive(Debug)]
pub struct StatePayload_String {
    pub body: String,
    pub hsbk: PayloadHSBK_String,
}

#[derive(Debug)]
pub struct PayloadHSBK {
    pub hue: u16,
    pub saturation: u16,
    pub brightness: u16,
    pub kelvin: u16,
}

#[derive(Debug)]
pub struct PayloadHSBK_String {
    pub hue: String,
    pub saturation: String,
    pub brightness: String,
    pub kelvin: String,
}

fn parse_payload_3(resp: &ResponseData) -> Payload {
    Payload::StateService(StateServicePayload {
        service: ResponseData::service(&resp),
        port: ResponseData::port(&resp),
        unknown: ResponseData::unknown(&resp),
    })
}
fn parse_payload_3_string(resp: &ResponseMessage) -> PayloadString {
    PayloadString::StateService(StateServicePayloadString {
        service: ResponseMessage::service(&resp),
        port: ResponseMessage::port(&resp),
        unknown: ResponseMessage::unknown(&resp),
    })
}

fn parse_payload_107(resp: &ResponseData) -> Payload {
    Payload::State(StatePayload {
        body: ResponseData::body(&resp),
        hsbk: PayloadHSBK {
            hue: ResponseData::hue(&resp),
            saturation: ResponseData::saturation(&resp),
            brightness: ResponseData::brightness(&resp),
            kelvin: ResponseData::kelvin(&resp),
        },
    })
}

fn parse_payload_107_string(resp: &ResponseMessage) -> PayloadString {
    PayloadString::State_String(StatePayload_String {
        body: ResponseMessage::body(&resp),
        hsbk: PayloadHSBK_String {
            hue: ResponseMessage::hue(&resp),
            saturation: ResponseMessage::saturation(&resp),
            brightness: ResponseMessage::brightness(&resp),
            kelvin: ResponseMessage::kelvin(&resp),
        },
    })
}

pub struct ResponseData(pub Vec<u8>);
pub struct ResponseMessage(pub Vec<u8>);

impl ResponseData {
    fn size(resp: &ResponseData) -> u16 {
        let mut b = extract(&resp, 0, 2);
        b.reverse();
        as_base10(b)
    }

    fn source(resp: &ResponseData) -> u32 {
        let mut b = extract(&resp, 4, 4);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr)
    }

    fn mac_address(resp: &ResponseData) -> String {
        as_hex(extract(&resp, 8, 8))
    }

    fn firmware(resp: &ResponseData) -> String {
        as_ascii(extract(&resp, 16, 6))
    }

    fn sequence_number(resp: &ResponseData) -> u16 {
        as_base10(extract(&resp, 23, 1))
    }

    fn reserved_1(resp: &ResponseData) -> u32 {
        let mut b = extract(&resp, 24, 8);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr)
    }

    fn message_type(resp: &ResponseData) -> u16 {
        let mut b = extract(&resp, 32, 2);
        b.reverse();
        as_base10(b)
    }

    fn reserved_2(resp: &ResponseData) -> u16 {
        let b = extract(&resp, 34, 2);
        as_base10(b)  // TODO: may not be base10, but undocumented.
    }

    fn service(resp: &ResponseData) -> u16 {
        as_base10(extract(&resp, 36, 1))
    }

    fn port(resp: &ResponseData) -> u32 {
        let mut b = extract(resp, 37, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr)
    }

    fn unknown(resp: &ResponseData) -> String {
        let end = resp.0.len() - 39;
        let b = extract(resp, 39, end);
        // as_base10(b)  // TODO: may not be base10, but undocumented.
        as_hex(b)
    }

    fn body(resp: &ResponseData) -> String {
        let end = resp.0.len() - 36;
        as_hex(extract(resp, 36, end))
    }

    fn hue(resp: &ResponseData) -> u16 {
        let mut b = extract(resp, 36, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u16(&bstr)
    }

    fn saturation(resp: &ResponseData) -> u16 {
        let mut b = extract(resp, 38, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u16(&bstr)
    }

    fn brightness(resp: &ResponseData) -> u16 {
        let mut b = extract(resp, 40, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u16(&bstr)
    }

    fn kelvin(resp: &ResponseData) -> u16 {
        let mut b = extract(resp, 42, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u16(&bstr)
    }
}

impl ResponseMessage {
    fn size(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 0, 2);
        b.reverse();
        as_base10_string(b)
    }

    fn source(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 4, 4);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn mac_address(resp: &ResponseMessage) -> String {
        as_hex(extract_string(resp, 8, 8))
    }

    fn firmware(resp: &ResponseMessage) -> String {
        as_ascii(extract_string(resp, 16, 6))
    }

    fn sequence_number(resp: &ResponseMessage) -> String {
        as_base10_string(extract_string(resp, 23, 1))
    }

    fn reserved_1(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 24, 8);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn message_type(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 32, 2);
        b.reverse();
        as_base10_string(b)
    }

    fn reserved_2(resp: &ResponseMessage) -> String {
        let b = extract_string(resp, 34, 2);
        as_base10_string(b)  // TODO: may not be base10, but undocumented.
    }

    fn service(resp: &ResponseMessage) -> String {
        as_base10_string(extract_string(resp, 36, 1))
    }

    fn port(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 37, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn unknown(resp: &ResponseMessage) -> String {
        let end = resp.0.len() - 39;
        let b = extract_string(resp, 39, end);
        // as_base10(b)  // TODO: may not be base10, but undocumented.
        as_hex(b)
    }

    fn body(resp: &ResponseMessage) -> String {
        let end = resp.0.len() - 36;
        as_hex(extract_string(&resp, 36, end))
    }

    fn hue(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 36, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn saturation(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 38, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn brightness(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 40, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }

    fn kelvin(resp: &ResponseMessage) -> String {
        let mut b = extract_string(resp, 42, 2);
        b.reverse();
        let bstr = as_boolean(b);
        bitstr_to_u32(&bstr).to_string()
    }
}

fn extract(resp: &ResponseData, start: usize, len: usize) -> Vec<u8> {
    let mut sub = vec![0u8; len];
    sub[..len].clone_from_slice(&resp.0[start..start + len]);
    sub
}

fn extract_string(resp: &ResponseMessage, start: usize, len: usize) -> Vec<u8> {
    let mut sub = vec![0u8; len];
    sub[..len].clone_from_slice(&resp.0[start..start + len]);
    sub
}

fn as_base10_string(v: Vec<u8>) -> String {
    let mut s = "".to_string();
    for b in v {
        s.push_str(format!("{}", b).as_str());
    }
    let n = s.parse::<u16>().unwrap();
    n.to_string()
}

fn as_base10(v: Vec<u8>) -> u16 {
    let mut s = "".to_string();
    for b in v {
        s.push_str(format!("{}", b).as_str());
    }
    s.parse::<u16>().unwrap()
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

fn bitstr_to_u8(bits: &str) -> u8 {
    bits.as_bytes().iter().fold(0, |acc, b| (acc << 1) + if *b == 48 { 0 } else { 1 })
}

fn bitstr_to_u16(bits: &str) -> u16 {
    bits.as_bytes().iter().fold(0, |acc, b| (acc << 1) + if *b == 48 { 0 } else { 1 })
}

fn bitstr_to_u32(bits: &str) -> u32 {
    bits.as_bytes().iter().fold(0, |acc, b| (acc << 1) + if *b == 48 { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::{ResponseData, ResponseMessage, extract, as_base10, as_ascii, as_boolean, as_hex,
                bitstr_to_u32};

    #[test]
    fn test_extract() {
        let resp = ResponseData(vec![41, 42, 43, 44, 45, 46, 47, 48, 49]);
        assert_eq!(extract(&resp, 2, 3), vec![43, 44, 45]);
    }

    #[test]
    fn test_as_base10() {
        assert_eq!(as_base10(vec![00, 41]), 41);
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
