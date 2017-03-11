#![allow(dead_code)]

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::str;

use colour;
use response;
use response::Response;

type Bit = bool;

struct Request {
    header: Header,
    payload: Payload,
}

impl Request {
    pub fn new(header: Header, payload: Payload) -> Request {
        Request {
            header: header,
            payload: payload,
        }
    }
}

struct Payload(Vec<u8>);

// RequestBin newtype
pub struct RequestBin(Vec<u8>);

struct Header {
    frame: Frame,
    frame_address: FrameAddress,
    protocol_header: ProtocolHeader,
}

impl Header {
    pub fn new(frame: Frame,
               frame_address: FrameAddress,
               protocol_header: ProtocolHeader)
               -> Header {
        Header {
            frame: frame,
            frame_address: frame_address,
            protocol_header: protocol_header,
        }
    }
}

struct Frame {
    // First 2 bytes
    size: u16,

    // Second two bytes
    origin: u8,
    // For discovery using Device::GetService use true and target all zeroes.
    // For all other requests set to false and target to device MAC address.
    tagged: bool,
    addressable: bool, // Must be true
    protocol: u16, // Must be 1024

    // Final 4 bytes
    source: u32,
}

impl Frame {
    pub fn new(origin: u8, tagged: bool, addressable: bool, protocol: u16, source: u32) -> Frame {
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
    pub fn new(target: [u8; 8],
               reserved: [u8; 6],
               reserved_2: u8,
               ack_required: bool,
               res_required: bool,
               sequence: u8)
               -> FrameAddress {
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
    pub fn new(reserved: u64, message_type: u16, reserved_2: u16) -> ProtocolHeader {
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
    // For all other requests set to false and target to device MAC address.
    tagged: Bit,
    addressable: Bit, // Must be true
    protocol: BitProtocol, // Must be 1024

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
        let v: Vec<Bit> = s.as_bytes()
            .iter()
            .map(|&n| if n == 49 { true } else { false })
            .collect();
        BitOrigin([v[0], v[1]])
    }
}

// BitProtocol newtype
struct BitProtocol([Bit; 12]);

impl From<u16> for BitProtocol {
    fn from(p: u16) -> BitProtocol {
        let s = format!("{:012b}", p);
        let v: Vec<Bit> = s.as_bytes()
            .iter()
            .map(|&n| if n == 49 { true } else { false })
            .collect();
        BitProtocol([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11]])
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
                let v: Vec<bool> = s.as_bytes()
                    .iter()
                    .map(|&n| if n == 49 { true } else { false })
                    .collect();
                let a: [Bit; 6] = [v[0], v[1], v[2], v[3], v[4], v[5]];
                a
            },
            ack_required: f.ack_required,
            res_required: f.res_required,
            sequence: f.sequence,
        }
    }
}

impl From<Request> for RequestBin {
    fn from(msg: Request) -> RequestBin {
        let mut msg_bin = RequestBin(vec![]);

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
            fr_pt2[i + 4] = bitframe.protocol.0[i];
        }

        let (fr_pt2_a_bits, fr_pt2_b_bits) = fr_pt2.split_at(8);
        let fr_pt2_a = RequestBin::bits_to_byte(fr_pt2_a_bits);
        let fr_pt2_b = RequestBin::bits_to_byte(fr_pt2_b_bits);

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

        let fa_pt2_byte = RequestBin::bits_to_byte(&fa_pt2);
        msg_bin.extend_with_u8(fa_pt2_byte);

        // Final byte of FrameAddress
        msg_bin.extend_with_u8(msg.header.frame_address.sequence);

        // First 8 bytes of ProtocolHeader
        msg_bin.extend_with_u64(msg.header.protocol_header.reserved);
        // Second, 2 bytes of ProtocolHeader
        msg_bin.extend_with_u16(msg.header.protocol_header.message_type);
        // Final 2 bytes of ProtocolHeader
        msg_bin.extend_with_u16(msg.header.protocol_header.reserved_2);

        // Append payload if required.
        if msg.header.protocol_header.message_type == 102 {
            msg_bin.extend_with_u8(msg.payload.0[..1][0]);
            let mut p_arr_h = [0u8; 2];
            let mut p_arr_s = [0u8; 2];
            let mut p_arr_b = [0u8; 2];
            let mut p_arr_k = [0u8; 2];
            let mut p_arr_d = [0u8; 4];
            p_arr_h.clone_from_slice(&msg.payload.0[1..3]);
            p_arr_s.clone_from_slice(&msg.payload.0[3..5]);
            p_arr_b.clone_from_slice(&msg.payload.0[5..7]);
            p_arr_k.clone_from_slice(&msg.payload.0[7..9]);
            p_arr_d.clone_from_slice(&msg.payload.0[9..13]);
            msg_bin.extend_with_u8_array_2(p_arr_h);
            msg_bin.extend_with_u8_array_2(p_arr_s);
            msg_bin.extend_with_u8_array_2(p_arr_b);
            msg_bin.extend_with_u8_array_2(p_arr_k);
            msg_bin.extend_with_u8_array_4(p_arr_d);
        }

        // Set message size in first 2 bytes of request, Frame.
        let mut p = RequestBin::u16_to_u8_array(msg_bin.0.len() as u16);
        p.reverse();
        msg_bin.0[0] = p[0];
        msg_bin.0[1] = p[1];
        msg_bin
    }
}

impl RequestBin {
    fn bits_to_byte(bits: &[Bit]) -> u8 {
        bits.iter().fold(0, |acc, b| (acc << 1) + if *b { 1 } else { 0 })
    }

    fn extend_with_bool(&mut self, field: bool) {
        // No need to reverse endianness, single byte.
        self.0.extend_from_slice(&RequestBin::bool_to_u8_array(field));
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

    fn extend_with_u8_array_4(&mut self, mut field: [u8; 4]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
        self.pp();
    }

    fn extend_with_u8_array_2(&mut self, mut field: [u8; 2]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
        self.pp();
    }

    fn extend_with_u16(&mut self, field: u16) {
        let mut p = RequestBin::u16_to_u8_array(field);
        p.reverse();
        self.pp();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn pp(&self) {
        return;
        // TODO: implement debug switch
        //
        // println!("Request: ");
        // for b in self.0.iter() {
        // print!("{:x} ", b);
        // }
        // println!("");
        //

    }

    fn extend_with_u32(&mut self, field: u32) {
        let mut p = RequestBin::u32_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn extend_with_u64(&mut self, field: u64) {
        let mut p = RequestBin::u64_to_u8_array(field);
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

    pub fn u16_to_u8_array(x: u16) -> [u8; 2] {
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

pub struct Network {}

pub struct Device {
    socket_addr: SocketAddr,
    pub response: Response,
}

impl Network {
    fn send_discover_devices(msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = true;
        let broadcast_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0));
        let broadcast_port = 56700;
        let broadcast_sock_addr = SocketAddr::new(broadcast_ip, broadcast_port);

        send(msg_bin, use_broadcast, broadcast_sock_addr)
    }
}

impl Device {
    fn send_get_device_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }

    fn send_set_device_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }
}

fn send(msg_bin: RequestBin,
        broadcast: bool,
        device_socket_addr: SocketAddr)
        -> Result<Device, io::Error> {
    let local_ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let local_sock_addr = SocketAddr::new(local_ip, 56700);
    let local_sock = try!(UdpSocket::bind(local_sock_addr));
    local_sock.set_broadcast(broadcast)?;

    let msg = &msg_bin.0;
    display(msg);
    try!(local_sock.send_to(&msg, device_socket_addr));

    // Read from the socket
    let mut resp_buf = [0; 1024];
    let (sz, src_sock_addr) = try!(local_sock.recv_from(&mut resp_buf));

    let resp_msg = &resp_buf[0..sz];
    println!("Received from {} : \n{:?}", src_sock_addr, resp_msg);

    let resp = response::parse_response(response::ResponseMessage(resp_msg.to_vec()));

    let device = Device {
        socket_addr: src_sock_addr,
        response: resp,
    };

    Ok(device)
}

fn display(msg_bin: &Vec<u8>) {
    println!("---- Sending request: ----");
    println!("Dec: {:?}", msg_bin);
    print!("Bytes: ");
    for b in msg_bin.iter() {
        print!("{:x} ", b);
    }
    println!("\n----");
}

pub fn get_service() -> Result<Device, io::Error> {
    let frame = Frame::new(0, true, true, 1024, 321);
    let frame_address = FrameAddress::new([0; 8], [0; 6], 0, false, false, 156);
    let protocol_header = ProtocolHeader::new(0, 2, 0);

    let header = Header::new(frame, frame_address, protocol_header);
    let payload = Payload(vec![]);
    let msg = Request::new(header, payload);
    let msg_bin = RequestBin::from(msg);

    let resp = match Network::send_discover_devices(msg_bin) {
        Ok(r) => {
            println!("good send");
            Ok(r)
        }
        Err(e) => {
            println!("bad send: {}", e);
            Err(e)
        }
    };
    resp
}

pub fn get_device_state(device: Device) -> Result<Device, io::Error> {
    let frame = Frame::new(0, false, true, 1024, 321);
    let frame_address = FrameAddress::new([0; 8], [0; 6], 0, false, false, 156);
    let protocol_header = ProtocolHeader::new(0, 101, 0);

    let header = Header::new(frame, frame_address, protocol_header);
    let payload = Payload(vec![]);
    let msg = Request::new(header, payload);
    let msg_bin = RequestBin::from(msg);

    let resp = match device.send_get_device_state(msg_bin) {
        Ok(r) => {
            println!("good send");
            Ok(r)
        }
        Err(e) => {
            println!("bad send: {}", e);
            Err(e)
        }
    };
    resp
}

pub fn set_device_state(device: &Device,
                        hsb: colour::HSB,
                        kelvin: u16,
                        duration: u32)
                        -> Result<Device, io::Error> {
    let frame = Frame::new(0, false, true, 1024, 321);
    let frame_address = FrameAddress::new([0; 8], [0; 6], 0, true, false, 156);
    let protocol_header = ProtocolHeader::new(0, 102, 0);

    let header = Header::new(frame, frame_address, protocol_header);

    // Payload example:
    // vec![0x00, 0xF7, 0x77, 0xFF, 0x0F, 0x4F, 0xFF, 0xA0, 0xAA, 0x00, 0x00, 0x03, 0xe8]

    let mut reserved = vec![0x00];
    let mut h = colour::hue_degrees_to_word(hsb.hue).to_vec();
    let mut s = colour::saturation_percent_to_word(hsb.saturation).to_vec();
    let mut b = colour::brightness_percent_to_word(hsb.brightness).to_vec();
    let mut k = RequestBin::u16_to_u8_array(kelvin).to_vec();
    let mut d = RequestBin::u32_to_u8_array(duration).to_vec();

    let mut payload_bytes = vec![];

    payload_bytes.append(&mut reserved);
    payload_bytes.append(&mut h);
    payload_bytes.append(&mut s);
    payload_bytes.append(&mut b);
    payload_bytes.append(&mut k);
    payload_bytes.append(&mut d);

    let payload = Payload(payload_bytes);

    let msg = Request::new(header, payload);
    let msg_bin = RequestBin::from(msg);

    let resp = match device.send_set_device_state(msg_bin) {
        Ok(r) => {
            println!("good send");
            Ok(r)
        }
        Err(e) => {
            println!("bad send: {}", e);
            Err(e)
        }
    };
    resp
}
