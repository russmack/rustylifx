use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use request::RequestBin;
use response::{self, Response};

const DEBUG_ENABLED: bool = false;

pub struct Network {}

/// Represents a device on the network, as well as a response.
pub struct Device {
    pub socket_addr: SocketAddr,
    pub response: Option<Response>,
}

impl Network {
    pub fn send_discover_devices(
        msg_bin: RequestBin,
        subnet: Ipv4Addr,
    ) -> Result<Device, io::Error> {
        let use_broadcast = true;

        // Ensure last octet is 255, broadcast.
        let broadcast_ip = ensure_ip_is_broadcast(subnet);

        let broadcast_port = 56700;
        let broadcast_sock_addr = SocketAddr::new(broadcast_ip, broadcast_port);

        send(msg_bin, use_broadcast, broadcast_sock_addr)
    }
}

fn ensure_ip_is_broadcast(subnet: Ipv4Addr) -> IpAddr {
    let octets = subnet.octets();
    match octets[3] {
        255 => IpAddr::V4(subnet),
        _ => IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], 255)),
    }
}

impl Device {
    pub fn send_get_device_power_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }

    pub fn send_set_device_power_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }

    pub fn send_get_device_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }

    pub fn send_set_device_state(&self, msg_bin: RequestBin) -> Result<Device, io::Error> {
        let use_broadcast = false;

        send(msg_bin, use_broadcast, self.socket_addr)
    }
}

fn send(
    msg_bin: RequestBin,
    broadcast: bool,
    device_socket_addr: SocketAddr,
) -> Result<Device, io::Error> {
    let local_ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let local_sock_addr = match broadcast {
        true => SocketAddr::new(local_ip, 0),
        false => SocketAddr::new(local_ip, 56700),
    };
    let local_sock = UdpSocket::bind(local_sock_addr)?;
    let _ = local_sock.set_write_timeout(Some(Duration::new(3, 0)));
    let _ = local_sock.set_read_timeout(Some(Duration::new(3, 0)));
    local_sock.set_broadcast(broadcast)?;

    let msg = &msg_bin.0;
    display(msg);
    print_debug("** sending...");
    match local_sock.send_to(msg, device_socket_addr) {
        Ok(v) => {
            print_debug(&format!("** sent {} bytes.", v));
        }
        Err(e) => {
            print_debug(&format!("** err sending: {}", e));
            return Err(e);
        }
    };

    // Read from the socket
    print_debug("** reading...");
    let mut resp_buf = [0; 1024];
    let (sz, src_sock_addr) = match local_sock.recv_from(&mut resp_buf) {
        Ok(v) => {
            print_debug("** response read done\n");
            v
        }
        Err(e) => {
            print_debug(&format!("** response read failed: {}", e));
            return Err(e);
        }
    };

    let resp_msg = &resp_buf[0..sz];
    print_debug(&format!(
        "Received from {} : \n{:?}",
        src_sock_addr, resp_msg
    ));

    let resp = response::parse_response(response::ResponseData(resp_msg.to_vec()));
    // let resp = response::parse_response(response::ResponseMessage(resp_msg.to_vec()));

    let device = Device {
        socket_addr: src_sock_addr,
        response: Some(resp),
    };

    Ok(device)
}

fn display(msg_bin: &[u8]) {
    print_debug("---- Sending request: ----\n");
    print_debug(&format!("Dec: {:?}\n", msg_bin));
    print_debug("Bytes: \n");
    for b in msg_bin.iter() {
        print_debug(&format!("{:x} ", b));
    }
    print_debug("\n----\n");
}

pub fn print_debug(s: &str) {
    if !DEBUG_ENABLED {
        return;
    }

    print!("{}", s);
}
