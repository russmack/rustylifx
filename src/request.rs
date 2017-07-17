#![allow(dead_code)]

use std::str;

type Bit = bool;

#[derive(Debug)]
pub struct Request {
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

#[derive(Debug)]
pub struct Payload(pub Vec<u8>);

// RequestBin newtype
pub struct RequestBin(pub Vec<u8>);

#[derive(Debug)]
pub struct Header {
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

#[derive(Debug)]
pub struct Frame {
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

#[derive(Debug)]
pub struct FrameAddress {
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

#[derive(Debug)]
pub struct ProtocolHeader {
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
        // Payload for 102.
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

        // Payload for 21.
        if msg.header.protocol_header.message_type == 21 {
            msg_bin.extend_with_u8(msg.payload.0[..1][0]);
            let mut p_arr_p = [0u8; 2];
            p_arr_p.clone_from_slice(&msg.payload.0[1..3]);
            msg_bin.extend_with_u8_array_2(p_arr_p);
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
    }

    fn extend_with_u8(&mut self, field: u8) {
        // No need to reverse endianness, single byte.
        self.0.extend_from_slice(&[field]);
    }

    fn extend_with_u8_array_8(&mut self, mut field: [u8; 8]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
    }

    fn extend_with_u8_array_6(&mut self, mut field: [u8; 6]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
    }

    fn extend_with_u8_array_4(&mut self, mut field: [u8; 4]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
    }

    fn extend_with_u8_array_2(&mut self, mut field: [u8; 2]) {
        field.reverse();
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
    }

    fn extend_with_u16(&mut self, field: u16) {
        let mut p = RequestBin::u16_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
    }

    fn extend_with_u32(&mut self, field: u32) {
        let mut p = RequestBin::u32_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
    }

    fn extend_with_u64(&mut self, field: u64) {
        let mut p = RequestBin::u64_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
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

    pub fn u32_to_u8_array(x: u32) -> [u8; 4] {
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

