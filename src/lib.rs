#![allow(dead_code)]

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

//use Endianness::{Big, Little};

enum Endianness {
    Big, 
    Little,
}

type Bit = bool;

struct Message {
    header: Header, 
    //payload: Payload,
}

struct Header {
    frame: Frame, 
    frame_address: FrameAddress, 
    protocol_header: ProtocolHeader, 
}

// BitFrame is an intermediate representation.
struct BitFrame {
    // First 2 bytes
    size: u16, 

    // Second two bytes
    origin: [Bit; 2], 
    // For discovery using Device::GetService use true and target all zeroes.
    // For all other messages set to false and target to device MAC address.
    tagged: Bit, 
    addressable: Bit,  // Must be true
    protocol: [Bit; 12],  // Must be 1024

    // Final 4 bytes
    source: u32,
}

impl<'a> From<&'a Frame> for BitFrame {
    fn from(f: &Frame) -> BitFrame {
        BitFrame {
            // First two bytes
            size: f.size, 

            // Second two bytes
            origin: {
                // Format as zero padded boolean string.
                let s = format!("{:02b}", f.origin);
                // Convert boolean string to vec of 1s and 0s.
                let v: Vec<bool> = s.as_bytes().iter().map(|&n| 
                    if n == 49 { true } 
                    else { false }).collect();
                let a: [Bit; 2] = [v[0], v[1]];
                a
            },
            tagged: f.tagged, 
            addressable: f.addressable, 
            protocol: {
               // Format as zero padded boolean string.
                let s = format!("{:012b}", f.protocol);
                // Convert boolean string to vec of 1s and 0s.
                let v: Vec<bool> = s.as_bytes().iter().map(|&n| 
                    if n == 49 { true } 
                    else { false }).collect();
                let a: [Bit; 12] = [
                    v[0], v[1], v[2], v[3],  v[4],  v[5],  v[6],  
                    v[7], v[8], v[9], v[10], v[11], 
                ];
                a
            },

            // Final four bytes
            source: f.source,
        }
    } 
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

struct FrameAddress {
    // MAC address (6 bytes) left-justified with two 0 bytes, or all 0s for all devices
    target: [u8; 8],  
    reserved: [u8; 6],
    reserved_2: u8,
    ack_required: bool,
    res_required: bool,
    sequence: u8,
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

struct ProtocolHeader {
    reserved: u64,
    message_type: u16,
    reserved_2: u16,
}

// TODO: This varies from message type to message type
struct Payload {}

#[derive(Debug)]
struct Packet(Vec<u8>);

//impl Packet {
impl Header {
    fn build(&mut self, msg: Message) {
        // First 2 bytes of Frame
        self.extend_with_u16(msg.header.frame.size);

        // Second 2 bytes of Frame
        let bitframe = BitFrame::from(&msg.header.frame);        
        
        let mut fr_pt2: [Bit; 16] = [false; 16];
        fr_pt2[0] = bitframe.origin[0];
        fr_pt2[1] = bitframe.origin[1];
        fr_pt2[2] = bitframe.tagged;
        fr_pt2[3] = bitframe.addressable; 
        for i in 0..bitframe.protocol.len() {
            fr_pt2[i+4] = bitframe.protocol[i];
        }

        let (fr_pt2_a_bits, fr_pt2_b_bits) = fr_pt2.split_at(8);
        let fr_pt2_a = Header::bits_to_byte(fr_pt2_a_bits);
        let fr_pt2_b = Header::bits_to_byte(fr_pt2_b_bits);

        // Add these two bytes in little endian order.
        self.extend_with_u8(fr_pt2_b);
        self.extend_with_u8(fr_pt2_a);

        // Final 4 bytes of Frame
        self.extend_with_u32(msg.header.frame.source);

        // First, 8 bytes of FrameAddress
        self.extend_with_u8_array_8(msg.header.frame_address.target);

        // Second, 6 bytes of FrameAddress
        self.extend_with_u8_array_6(msg.header.frame_address.reserved);

        // Third, 1 byte of FrameAddress
        let bitframeaddress = BitFrameAddress::from(&msg.header.frame_address);

        let mut fa_pt2: [Bit; 8] = [false; 8];
        let rlen = bitframeaddress.reserved_2.len();
        for i in 0..rlen {
            fa_pt2[i] = bitframeaddress.reserved_2[i];
        }
        fa_pt2[rlen + 0] = bitframeaddress.ack_required;
        fa_pt2[rlen + 1] = bitframeaddress.res_required;

        let fa_pt2_byte = Header::bits_to_byte(&fa_pt2);
        self.extend_with_u8(fa_pt2_byte);

        // Final byte of FrameAddress
        self.extend_with_u8(msg.header.frame_address.sequence);

        // First 8 bytes of ProtocolHeader
        self.extend_with_u64(msg.header.protocol_header.reserved);
        // Second, 2 bytes of ProtocolHeader
        self.extend_with_u16(msg.header.protocol_header.message_type);
        // Final 2 bytes of ProtocolHeader
        self.extend_with_u16(msg.header.protocol_header.reserved_2);

        // Set packet size in first 2 bytes of packet, Frame.
        let mut p = Header::u16_to_u8_array(self.len() as u16);
        //let mut p = Header::u16_to_u8_array(self.0.len() as u16);
        p.reverse();
        self.0[0] = p[0];
        self.0[1] = p[1];
    }

    fn bits_to_byte(bits: &[Bit]) -> u8 {
        bits.iter().fold(0, |acc, b| (acc << 1) + if *b { 1 } else { 0 } )
    }

    fn extend_with_bool(&mut self, field: bool) {
        // No need to reverse endianness, single byte.
        self.0.extend_from_slice(&Packet::bool_to_u8_array(field));
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
        let mut p = Packet::u16_to_u8_array(field);
        p.reverse();
        self.pp();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn pp(&self) {
        return;  
        // TODO: implement debug switch
        /*
        println!("Packet: ");
        for b in self.0.iter() {
            print!("{:x} ", b);
        }
        println!("");
        */
    }

    fn extend_with_u32(&mut self, field: u32) {
        let mut p = Packet::u32_to_u8_array(field);
        p.reverse();
        self.0.extend_from_slice(&p);
        self.pp();
    }

    fn extend_with_u64(&mut self, field: u64) {
        let mut p = Packet::u64_to_u8_array(field);
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

fn send(msg: Message) -> Result<(), io::Error> {
    let dev = true;
    let (local_ip, remote_ip) = match dev {
        true => (Ipv4Addr::new(127, 0, 0, 1), Ipv4Addr::new(127, 0, 0, 1)), 
        false => (Ipv4Addr::new(192, 168, 0, 2), Ipv4Addr::new(192, 168, 0, 5)), 
    };

    //let local_ip = Ipv4Addr::new(192, 168, 0, 2);
    //let local_ip = Ipv4Addr::new(127,0,0,1);
    let conn = SocketAddrV4::new(local_ip, 56700);
    let socket = try!(UdpSocket::bind(conn));

    let mut buf = [0; 100]; // for recv

    //let remote_ip = Ipv4Addr::new(127,0,0,1);
    //let remote_ip = Ipv4Addr::new(192, 168, 0, 5);
    let remote_conn = SocketAddrV4::new(remote_ip, 56700);

    socket.set_broadcast(true)?;
    
    //let mut packet: Packet = Packet(vec![0u8; 0]);
    let mut header: Header = Header::new();
    header.build(msg);
    let packet: Packet = Packet { Header: header };
    
    let p = &packet.0;
    println!("Dec: {:?}", p);

    println!("---- Sending packet: ----");
        for b in p.iter() {
            print!("{:x} ", b);
        }
    println!("\n----");

    try!(socket.send_to(&p, remote_conn));

    // Read from the socket
    let (amt, src) = try!(socket.recv_from(&mut buf));

    let resp_packet = &buf[0..amt];
    println!("Received from {} : {:?}", src, resp_packet);

    parse_response(resp_packet.to_vec());
    // let resp_obj = parse_response(resp);
    // display_response(resp_obj);

    Ok(())
}

fn parse_response(resp: Vec<u8>) {
    // let mut res = StateServiceMessage {
    //     Header: 
    // };
    println!("Parse: {:?}", resp);
}

fn all_le(v: Vec<u8>) -> Vec<u8> {
    // TODO: implement
    let mut le_vec: Vec<u8> = vec![];
    for b in v {
        le_vec.push(b.to_le());
    }
    le_vec
}

pub fn get_service() {
    let msg = Message {
        frame: Frame {
            size: 0, 
            origin: 0, 
            tagged: true, 
            addressable: true,
            protocol: 1024, 
            source: 321,
        },
        frame_address: FrameAddress {
            //target: [0x31, 0x19, 0x95, 0x4c, 0xb9, 0xbc, 0x00, 0x00],  
            target: [0; 8], 
            reserved: [0; 6],
            reserved_2: 0,
            ack_required: false,
            res_required: false, 
            sequence: 156,
        },
        protocol_header: ProtocolHeader {
            reserved: 0,
            message_type: 2,
            reserved_2: 0,
        },
    };

    match send(msg) {
        Ok(()) => println!("good send"),
        Err(e) => println!("bad send: {}", e),
    };
}

fn get_device_state() {
    let _ = Message {
        frame: Frame {
            size: 0, 
            origin: 0, 
            tagged: false, 
            addressable: true,
            protocol: 1024, 
            source: 321,
        },
        frame_address: FrameAddress {
            target: [0; 8], 
            reserved: [0; 6],
            reserved_2: 0,
            ack_required: false,
            res_required: false, 
            sequence: 156,
        },
        protocol_header: ProtocolHeader {
            reserved: 0,
            message_type: 101,
            reserved_2: 0,
        },
    };
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
