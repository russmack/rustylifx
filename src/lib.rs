#![allow(dead_code)]

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::str;

type Bit = bool;

struct Message {
    header: Header, 
    payload: Payload,
}

impl Message {
    pub fn new(header: Header, payload: Payload) -> Message {
        Message {header: header, payload: payload}
    }
}

// MessageBin newtype
struct MessageBin(Vec<u8>);

struct Header {
    frame: Frame, 
    frame_address: FrameAddress, 
    protocol_header: ProtocolHeader, 
}

impl Header {
    pub fn new( frame: Frame, 
                frame_address: FrameAddress, 
                protocol_header: ProtocolHeader) -> Header {
        Header {frame: frame, 
                frame_address: frame_address, 
                protocol_header: protocol_header, 
        }
    }
}

// TODO: This varies from message type to message type
// Payload newtype
struct Payload(Vec<u8>);

impl Payload {
    pub fn new() -> Payload {
        Payload(vec![])
    }
}

struct Frame {
    // First 2 bytes
    size: u16, 

    // Second two bytes
    origin: u8, 
    // For discovery using Device::GetService use true and target all zeroes.
    // For all other messages set to false and target to device MAC address.
    tagged: bool, 
    addressable: bool,  // Must be true
    protocol: u16,  // Must be 1024

    // Final 4 bytes
    source: u32,
}

impl Frame {
    pub fn new( origin: u8, 
                tagged: bool, 
                addressable: bool, 
                protocol: u16, 
                source: u32) -> Frame {
        Frame {
            size: 0, 
            origin: origin, 
            tagged: tagged, 
            addressable: addressable, 
            protocol: protocol, 
            source: source, 
        }
    }
}

struct FrameAddress {
    // MAC address (6 bytes) left-justified with two 0 bytes, or all 0s for all devices
    target: [u8; 8],  
    reserved: [u8; 6],
    reserved_2: u8,
    ack_required: bool,
    res_required: bool,
    sequence: u8,
}

impl FrameAddress {
    pub fn new( target: [u8; 8], 
                reserved: [u8; 6], 
                reserved_2: u8, 
                ack_required: bool, 
                res_required: bool, 
                sequence: u8) -> FrameAddress {
        FrameAddress {
            target: target, 
            reserved: reserved, 
            reserved_2: reserved_2, 
            ack_required: ack_required, 
            res_required: res_required, 
            sequence: sequence, 
        }
    }
}

struct ProtocolHeader {
    reserved: u64,
    message_type: u16,
    reserved_2: u16,
}

impl ProtocolHeader {
    pub fn new( reserved: u64, 
                message_type: u16, 
                reserved_2: u16) -> ProtocolHeader {
        ProtocolHeader {
            reserved: reserved, 
            message_type: message_type, 
            reserved_2: reserved_2,
        }
    }
}

// BitFrame is an intermediate representation.
struct BitFrame {
    // First 2 bytes
    size: u16, 

    // Second two bytes
    origin: BitOrigin, 

    // For discovery using Device::GetService use true and target all zeroes.
    // For all other messages set to false and target to device MAC address.
    tagged: Bit, 
    addressable: Bit,       // Must be true
    protocol: BitProtocol,  // Must be 1024

    // Final 4 bytes
    source: u32,
}

impl<'a> From<&'a Frame> for BitFrame {
    fn from(f: &Frame) -> BitFrame {
        BitFrame {
            // First two bytes
            size: f.size, 

            // Second two bytes
            origin: BitOrigin::from(f.origin),
            tagged: f.tagged, 
            addressable: f.addressable, 
            protocol: BitProtocol::from(f.protocol),

            // Final four bytes
            source: f.source,
        }
    } 
}

// BitOrigin newtype
struct BitOrigin([Bit; 2]);

impl From<u8> for BitOrigin {
    // Format as zero padded boolean string.
    // Convert boolean string to vec of bools
    fn from(o: u8) -> BitOrigin {
        let s = format!("{:02b}", o);  
        let v: Vec<Bit> = s.as_bytes().iter().map(
            |&n|
                if n == 49 { true }
                else { false }
            ).collect();
        BitOrigin([v[0], v[1]])
    }
}

// BitProtocol newtype
struct BitProtocol([Bit; 12]);

impl From<u16> for BitProtocol {
    fn from(p: u16) -> BitProtocol {
        let s = format!("{:012b}", p);
        let v: Vec<Bit> = s.as_bytes().iter().map(
            |&n|
                if n == 49 { true }
                else { false }
            ).collect();
        BitProtocol([
            v[0], v[1], v[2], v[3],  v[4],  v[5],  
            v[6], v[7], v[8], v[9], v[10], v[11], 
        ])
    }
}


struct BitFrameAddress {
    // MAC address (6 bytes) left-justified with two 0 bytes, or all 0s for all devices
    target: [u8; 8],  
    reserved: [u8; 6],
    reserved_2: [Bit; 6],
    ack_required: Bit,
    res_required: Bit,
    sequence: u8,
}

impl<'a> From<&'a FrameAddress> for BitFrameAddress {
    fn from(f: &FrameAddress) -> BitFrameAddress {
        BitFrameAddress {
            target: f.target, 
            reserved: f.reserved, 
            reserved_2: {
                let s = format!("{:06b}", f.reserved_2);
                let v: Vec<bool> = s.as_bytes().iter().map(|&n|
                    if n == 49 { true }
                    else { false }).collect();
                let a: [Bit; 6] = [
                    v[0], v[1], v[2], v[3],  v[4],  v[5],
                ];
                a
            }, 
            ack_required: f.ack_required, 
            res_required: f.res_required,
            sequence: f.sequence,
        }
    }
}

impl From<Message> for MessageBin {
    fn from(msg: Message) -> MessageBin {
        let mut msg_bin = MessageBin(vec![]);

        // First 2 bytes of Frame
        msg_bin.extend_with_u16(msg.header.frame.size);

        // Second 2 bytes of Frame
        let bitframe = BitFrame::from(&msg.header.frame);        
        
        let mut fr_pt2: [Bit; 16] = [false; 16];
        fr_pt2[0] = bitframe.origin.0[0];
        fr_pt2[1] = bitframe.origin.0[1];
        fr_pt2[2] = bitframe.tagged;
        fr_pt2[3] = bitframe.addressable; 
        for i in 0..bitframe.protocol.0.len() {
            fr_pt2[i+4] = bitframe.protocol.0[i];
        }

        let (fr_pt2_a_bits, fr_pt2_b_bits) = fr_pt2.split_at(8);
        let fr_pt2_a = MessageBin::bits_to_byte(fr_pt2_a_bits);
        let fr_pt2_b = MessageBin::bits_to_byte(fr_pt2_b_bits);

        // Add these two bytes in little endian order.
        msg_bin.extend_with_u8(fr_pt2_b);
        msg_bin.extend_with_u8(fr_pt2_a);

        // Final 4 bytes of Frame
        msg_bin.extend_with_u32(msg.header.frame.source);

        // First, 8 bytes of FrameAddress
        msg_bin.extend_with_u8_array_8(msg.header.frame_address.target);

        // Second, 6 bytes of FrameAddress
        msg_bin.extend_with_u8_array_6(msg.header.frame_address.reserved);

        // Third, 1 byte of FrameAddress
        let bitframeaddress = BitFrameAddress::from(&msg.header.frame_address);

        let mut fa_pt2: [Bit; 8] = [false; 8];
        let rlen = bitframeaddress.reserved_2.len();
        for i in 0..rlen {
            fa_pt2[i] = bitframeaddress.reserved_2[i];
        }
        fa_pt2[rlen + 0] = bitframeaddress.ack_required;
        fa_pt2[rlen + 1] = bitframeaddress.res_required;

        let fa_pt2_byte = MessageBin::bits_to_byte(&fa_pt2);
        msg_bin.extend_with_u8(fa_pt2_byte);

        // Final byte of FrameAddress
        msg_bin.extend_with_u8(msg.header.frame_address.sequence);

        // First 8 bytes of ProtocolHeader
        msg_bin.extend_with_u64(msg.header.protocol_header.reserved);
        // Second, 2 bytes of ProtocolHeader
        msg_bin.extend_with_u16(msg.header.protocol_header.message_type);
        // Final 2 bytes of ProtocolHeader
        msg_bin.extend_with_u16(msg.header.protocol_header.reserved_2);

        // Set message size in first 2 bytes of message, Frame.
        let mut p = MessageBin::u16_to_u8_array(msg_bin.0.len() as u16);
        p.reverse();
        msg_bin.0[0] = p[0];
        msg_bin.0[1] = p[1];
        msg_bin
    }
}

impl MessageBin {
    fn bits_to_byte(bits: &[Bit]) -> u8 {
        bits.iter().fold(0, |acc, b| (acc << 1) + if *b { 1 } else { 0 } )
    }

    fn extend_with_bool(&mut self, field: bool) {
        // No need to reverse endianness, single byte.
        self.0.extend_from_slice(&MessageBin::bool_to_u8_array(field));
        self.pp();
    }

    fn extend_with_u8(&mut self, field: u8) {
        // No need to reverse endianness, single byte.
        self.0.extend_from_slice(&[field]);
        self.pp();
    }

    fn extend_with_u8_array_8(&mut self, mut field: [u8; 8]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
        self.pp();
    }

    fn extend_with_u8_array_6(&mut self, mut field: [u8; 6]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
        self.pp();
    }

    fn extend_with_u16(&mut self, field: u16) {
        let mut p = MessageBin::u16_to_u8_array(field);
        p.reverse();
        self.pp();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn pp(&self) {
        return;  
        // TODO: implement debug switch
        /*
        println!("Message: ");
        for b in self.0.iter() {
            print!("{:x} ", b);
        }
        println!("");
        */
    }

    fn extend_with_u32(&mut self, field: u32) {
        let mut p = MessageBin::u32_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn extend_with_u64(&mut self, field: u64) {
        let mut p = MessageBin::u64_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn bool_to_u8_array(b: bool) -> [u8; 1] {
        match b {
            true => [1],
            false => [0],
        }
    }

    fn u16_to_u8_array(x: u16) -> [u8; 2] {
        let b1: u8 = ((x >> 8) & 0xff) as u8;
        let b2: u8 = (x & 0xff) as u8;
        [b1, b2]
    }

    fn u32_to_u8_array(x: u32) -> [u8; 4] {
        let b1: u8 = ((x >> 24) & 0xff) as u8;
        let b2: u8 = ((x >> 16) & 0xff) as u8;
        let b3: u8 = ((x >> 8) & 0xff) as u8;
        let b4: u8 = (x & 0xff) as u8;
        [b1, b2, b3, b4]
    }

    fn u64_to_u8_array(x: u64) -> [u8; 8] {
        let b1: u8 = ((x >> 56) & 0xff) as u8;
        let b2: u8 = ((x >> 48) & 0xff) as u8;
        let b3: u8 = ((x >> 40) & 0xff) as u8;
        let b4: u8 = ((x >> 32) & 0xff) as u8;
        let b5: u8 = ((x >> 24) & 0xff) as u8;
        let b6: u8 = ((x >> 16) & 0xff) as u8;
        let b7: u8 = ((x >> 8) & 0xff) as u8;
        let b8: u8 = (x & 0xff) as u8;
        [b1, b2, b3, b4, b5, b6, b7, b8]
    }
}

fn send(msg_bin: MessageBin) -> Result<(), io::Error> {
    let dev = false;
    let (local_ip, remote_ip) = match dev {
        true => (Ipv4Addr::new(127, 0, 0, 1), Ipv4Addr::new(127, 0, 0, 1)), 
        false => (Ipv4Addr::new(192, 168, 0, 2), Ipv4Addr::new(192, 168, 0, 5)), 
    };

    let conn = SocketAddrV4::new(local_ip, 56700);
    let socket = try!(UdpSocket::bind(conn));
    let mut buf = [0; 1024]; // for recv

    let remote_conn = SocketAddrV4::new(remote_ip, 56700);
    socket.set_broadcast(true)?;
    
    let mb = &msg_bin.0;
    println!("Dec: {:?}", mb);

    println!("---- Sending message: ----");
    for b in mb.iter() {
        print!("{:x} ", b);
    }
    println!("\n----");

    try!(socket.send_to(&mb, remote_conn));

    // Read from the socket
    let (amt, src) = try!(socket.recv_from(&mut buf));

    let resp_msg = &buf[0..amt];
    println!("Received from {} : {:?}", src, resp_msg);

    let resp = parse_response(ResponseMessage(resp_msg.to_vec()));
    display_response(resp);

    Ok(())
}

fn display_response(resp: Response) {
    println!("Response:");
    println!("Size: {}", resp.size);
    println!("Source: {:?}", resp.source);
    println!("Mac addr: {:?}", resp.mac_address);
    println!("Firmware: {:?}", resp.firmware);
    // packed byte
    println!("Sequence num: {:?}", resp.sequence_number);

    println!("Reserved_1 (timestamp?): {:?}", resp.reserved_1);
    
    println!("Message type: {:?}", resp.message_type);

    println!("Reserved_2: {:?}", resp.reserved_2);

    println!("Service: {:?}", resp.service);
    println!("Port: {:?}", resp.port);
    println!("Unknown: {:?}", resp.unknown);
    
}

struct Response {
    size: String, 
    source: String, 
    mac_address: String,
    firmware: String,    
    sequence_number: String, 
    reserved_1: String,
    message_type: String, 
    reserved_2: String, 
    service: String, 
    port: String, 
    unknown: String, 
}

fn parse_response(resp: ResponseMessage) -> Response {
    let mut response = Response {
        size: "".to_string(),
        source: "".to_string(), 
        mac_address: "".to_string(), 
        firmware: "".to_string(), 
        sequence_number: "".to_string(), 
        reserved_1: "".to_string(),
        message_type: "".to_string(), 
        reserved_2: "".to_string(), 
        service: "".to_string(), 
        port: "".to_string(),
        unknown: "".to_string(), 
    };

    response.size = ResponseMessage::size(&resp);
    response.source = ResponseMessage::source(&resp);
    response.mac_address = ResponseMessage::mac_address(&resp);
    response.firmware = ResponseMessage::firmware(&resp);

    // TODO: packed byte

    response.sequence_number = ResponseMessage::sequence_number(&resp);

    // Message segment: protocol header
    response.reserved_1 = ResponseMessage::reserved_1(&resp);  // timestamp?
    response.message_type = ResponseMessage::message_type(&resp);
    response.reserved_2 = ResponseMessage::reserved_2(&resp);
    // Message segment: payload
    response.service = ResponseMessage::service(&resp);
    response.port = ResponseMessage::port(&resp);
    response.unknown = ResponseMessage::unknown(&resp);

    response
}

struct ResponseMessage(Vec<u8>);

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
        let mut b = extract(&resp, 34, 2);
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
        let mut b = extract(&resp, 39, 2);
        as_base10(b)  // TODO: may not be base10, but undocumented.
    }
}

fn extract(resp: &ResponseMessage, start: usize, len: usize) -> Vec<u8> {
    let mut sub = vec![0u8; len];
    sub[..len].clone_from_slice(&resp.0[start..start+len]);
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

fn array_segment_to_string(arr: &[u8], lbound: usize, ubound: usize, reverse: bool) -> String {
    let len: usize = ubound - lbound;
    let mut sub = vec![0u8; len];
    sub[..len].clone_from_slice(&arr[lbound..ubound]);

    if reverse {
        sub.reverse();
    }

    String::from_utf8_lossy(&sub).into_owned()
}

fn bitstr_to_u32(bits: &str) -> u32 {
    bits.as_bytes().iter().fold(0, |acc, b | {
        (acc << 1) + if *b == 48 { 0 } else { 1 }
    })
}

#[cfg(test)]
mod tests {
    use super:: {
        ResponseMessage,
        extract,
        as_base10,
        as_ascii,
        as_boolean,
        as_hex,
        bitstr_to_u32,
    };

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
        assert_eq!(as_hex(vec![209, 114, 214, 20, 224, 14, 0, 0]), "D1:72:D6:14:E0:0E:00:00");
    }
}

pub fn get_service() {
    let frame = Frame::new(
        0,     // origin:
        true,  // tagged:
        true,  // addressable:
        1024,  // protocol:
        321,   // source:
    );
    let frame_address = FrameAddress::new(
        //target: [0x31, 0x19, 0x95, 0x4c, 0xb9, 0xbc, 0x00, 0x00],  
        [0; 8], // target:
        [0; 6], // reserved:
        0,      // reserved_2:
        false,  // ack_required:
        false,  // res_required:
        156,    // sequence:
    );
    let protocol_header = ProtocolHeader::new(
        0,      // reserved:
        2,      // message_type:
        0,      // reserved_2:
    );

    let header = Header::new( frame, frame_address, protocol_header );
    let payload = Payload::new();
    let msg = Message::new(header, payload);
    let msg_bin = MessageBin::from(msg);
    
    match send(msg_bin) {
        Ok(()) => println!("good send"),
        Err(e) => println!("bad send: {}", e),
    };
}

fn get_device_state() {
    let frame = Frame::new(
        0,      // origin:
        false,  // tagged:
        true,   // addressable:
        1024,   // protocol:
        321,    // source:
    );
    let frame_address = FrameAddress::new(
        [0; 8], // target:
        [0; 6], // reserved:
        0,      // reserved_2:
        false,  // ack_required:
        false,  // res_required:
        156,    // sequence:
    );
    let protocol_header = ProtocolHeader::new(
        0,      // reserved:
        101,    // message_type:
        0,      // reserved_2:
    );

    let header = Header::new( frame, frame_address, protocol_header );
    let payload = Payload::new();
    let msg = Message::new(header, payload);
    let msg_bin = MessageBin::from(msg);
    
    match send(msg_bin) {
        Ok(()) => println!("good send"),
        Err(e) => println!("bad send: {}", e),
    };
}

