use std::io;

use colour;
use network;
use request::Frame;
use request::FrameAddress;
use request::Header;
use request::Payload;
use request::ProtocolHeader;
use request::Request;
use request::RequestBin;

/// Finds devices on the network.
pub fn get_service() -> Result<network::Device, io::Error> {
    let msg = 
        Request::new(
            Header::new(
                Frame::new(0, true, true, 1024, 321),
                FrameAddress::new([0; 8], [0; 6], 0, false, false, 156),
                ProtocolHeader::new(0, 2, 0),
            ),
            Payload(vec![]),
        );

    let msg_bin = RequestBin::from(msg);

    let resp = match network::Network::send_discover_devices(msg_bin) {
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

/// Gets the state of the specified device.
pub fn get_device_state(device: network::Device) -> Result<network::Device, io::Error> {
    let msg = 
        Request::new(
            Header::new(
                Frame::new(0, false, true, 1024, 321),
                FrameAddress::new([0; 8], [0; 6], 0, false, false, 156),
                ProtocolHeader::new(0, 101, 0),
            ),
            Payload(vec![])
        );

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

/// Sets the state of the specified device.
pub fn set_device_state(device: &network::Device,
                        hsb: &colour::HSB,
                        kelvin: u16,
                        duration: u32)
                        -> Result<network::Device, io::Error> {

    //! # Sample payload:
    //! ```
    //! vec![0x00, 0xF7, 0x77, 0xFF, 0x0F, 0x4F, 0xFF, 0xA0, 0xAA, 0x00, 0x00, 0x03, 0xe8];
    //! ```

    let reserved = vec![0x00];
    let h = colour::hue_degrees_to_word(hsb.hue).to_vec();
    let s = colour::saturation_percent_to_word(hsb.saturation).to_vec();
    let b = colour::brightness_percent_to_word(hsb.brightness).to_vec();
    let k = RequestBin::u16_to_u8_array(kelvin).to_vec();
    let d = RequestBin::u32_to_u8_array(duration).to_vec();

    let payload_bytes = vec![
        &reserved[..],
        &h[..],
        &s[..],
        &b[..],
        &k[..],
        &d[..],
    ].concat();

    let msg = 
        Request::new(
            Header::new(
                Frame::new(0, false, true, 1024, 321),
                FrameAddress::new([0; 8], [0; 6], 0, true, false, 156),
                ProtocolHeader::new(0, 102, 0),
            ),
            Payload(payload_bytes),
        );

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
