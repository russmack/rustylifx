use std::io;
use std::net::{IpAddr, SocketAddr, UdpSocket, Ipv4Addr};

use request::RequestBin;
use response;
use response::Response;

pub struct Network {}

/// Represents a device on the network, as well as a response.
pub struct Device {
    socket_addr: SocketAddr,
    pub response: Response,
}

impl Network {
    pub fn send_discover_devices(msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = true;
        let broadcast_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0));
        let broadcast_port = 56700;
        let broadcast_sock_addr = SocketAddr::new(broadcast_ip, broadcast_port);

        send(msg_bin, use_broadcast, broadcast_sock_addr)
    }
}

impl Device {
    pub fn send_get_device_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }

    pub fn send_set_device_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
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

    let resp = response::parse_response(response::ResponseData(resp_msg.to_vec()));
    // let resp = response::parse_response(response::ResponseMessage(resp_msg.to_vec()));

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
