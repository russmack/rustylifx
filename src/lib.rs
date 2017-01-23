#![allow(dead_code)]

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};


struct Message {
    frame: Frame, 
    frame_address: FrameAddress, 
    protocol_header: ProtocolHeader, 
    //payload: Payload,
}

struct Frame {
    size: u16, 
    origin: u8, 
    // For discovery using Device::GetService use true and target all zeroes.
    // For all other messages set to false and target to device MAC address.
    tagged: bool, 
    addressable: bool,  // Must be true
    protocol: u16,  // Must be 1024
    source: u32,
}

struct FrameAddress {
    target: [u8; 8],  // MAC address, or 0 for all devices
    reserved: [u8; 6],
    reserved_2: u8,
    ack_required: bool,
    res_required: bool,
    sequence: u8,
}

struct ProtocolHeader {
    reserved: u64,
    message_type: u16,
    reserved_2: u16,
}

struct Payload {
    
}

#[derive(Debug)]
struct Packet(Vec<u8>);

impl Packet {
    fn build(&mut self, msg: Message) {
        self.extend_with_u16(msg.frame.size);
        self.extend_with_u8(msg.frame.origin);
        self.extend_with_bool(msg.frame.tagged);
        self.extend_with_bool(msg.frame.addressable);
        self.extend_with_u16(msg.frame.protocol);
        self.extend_with_u32(msg.frame.source);

        self.extend_with_u8_array_8(msg.frame_address.target);
        self.extend_with_u8_array_6(msg.frame_address.reserved);
        self.extend_with_u8(msg.frame_address.reserved_2);
        self.extend_with_bool(msg.frame_address.ack_required);
        self.extend_with_bool(msg.frame_address.res_required);
        self.extend_with_u8(msg.frame_address.sequence);

        self.extend_with_u64(msg.protocol_header.reserved);
        self.extend_with_u16(msg.protocol_header.message_type);
        self.extend_with_u16(msg.protocol_header.reserved_2);
    }

    fn extend_with_bool(&mut self, field: bool) {
        self.0.extend_from_slice(&Packet::bool_to_u8_array(field));
    }

    fn extend_with_u8(&mut self, field: u8) {
        self.0.extend_from_slice(&[field]);
    }

    fn extend_with_u8_array_8(&mut self, field: [u8; 8]) {
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
    }

    fn extend_with_u8_array_6(&mut self, field: [u8; 6]) {
        for b in field.iter() {
            self.0.extend_from_slice(&[*b]);
        }
    }

    fn extend_with_u16(&mut self, field: u16) {
        self.0.extend_from_slice(&Packet::u16_to_u8_array(field));
    }

    fn extend_with_u32(&mut self, field: u32) {
        self.0.extend_from_slice(&Packet::u32_to_u8_array(field));
    }

    fn extend_with_u64(&mut self, field: u64) {
        self.0.extend_from_slice(&Packet::u64_to_u8_array(field));
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
    let local_ip = Ipv4Addr::new(127, 0, 0, 1);
    let conn = SocketAddrV4::new(local_ip, 56700);
    let socket = try!(UdpSocket::bind(conn));

    let remote_ip = Ipv4Addr::new(192, 168, 0, 12);
    let remote_conn = SocketAddrV4::new(remote_ip, 56700);

    socket.set_broadcast(true);
    
    let mut packet: Packet = Packet(vec![0u8; 0]);
    packet.build(msg);
    println!("{:?}", packet);
    //try!(socket.send_to(packet, remote_conn));
    //try!(socket.send_to(msg.as_bytes(), remote_conn));
    
    Ok(())
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
            target: [0; 8],  // TODO: set to light's MAC address
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

    send(msg);
}

fn get_device_state() {
    let msg = Message {
        frame: Frame {
            size: 0, 
            origin: 0, 
            tagged: false, 
            addressable: true,
            protocol: 1024, 
            source: 321,
        },
        frame_address: FrameAddress {
            target: [0; 8],  // TODO: set to light's MAC address
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
