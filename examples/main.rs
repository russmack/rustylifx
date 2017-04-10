extern crate rustylifx;

use rustylifx::{colour, messages, response};
use rustylifx::network::Device;

use std::time::Duration;
use std::thread;

fn main() {
    let device = find_device();

    let device = get_device_state(device);

    parse_hsvk(&device);

    change_colour(device);

    println!("\nFinished.");
}

fn find_device() -> Device {
    let device = messages::get_service().unwrap();

    match device.response {
        Some(ref resp) => display_response("State service", &resp),
        None => panic!("no response"),
    };

    thread::sleep(Duration::from_millis(1000));
    device
}

fn get_device_state(device: Device) -> Device {
    let device = messages::get_device_state(device).unwrap();
    match device.response {
        Some(ref resp) => display_response("State", resp),
        None => panic!("no response"),
    };

    thread::sleep(Duration::from_millis(1000));
    device
}

fn parse_hsvk(device: &Device) {
    println!("\nCurrent state received:");
    let resp = match device.response {
        Some(ref v) => v,
        None => panic!("no response"),
    };

    let payload = match resp.payload {
        response::Payload::State(ref v) => Some(v),
        _ => None,
    };

    match payload {
        Some(v) => {
            println!("current payload body: {:?}", v.body);
            println!("current hue: {:?}", v.hsbk.hue);
            println!("current hue degrees: {:?}º",
                     colour::hue_word_to_degrees(v.hsbk.hue));
            println!("current sat: {:?}", v.hsbk.saturation);
            println!("current sat percent: {:?}%",
                     colour::saturation_word_to_percent(v.hsbk.saturation as u16));
            println!("current bri: {:?}", v.hsbk.brightness);
            println!("current bri percent: {:?}%",
                     colour::brightness_word_to_percent(v.hsbk.brightness as u16));
            println!("current kel: {:?}", v.hsbk.kelvin);
        }
        None => (),
    };
    println!("\n");
}

fn change_colour(device: Device) {
    // Use constants.
    let cols: Vec<colour::HSB> =
        vec![colour::get_colour("red"), colour::get_colour("green"), colour::get_colour("blue")];

    for c in cols {
        let _ = messages::set_device_state(&device, &c, 1000, 0);
        thread::sleep(Duration::from_millis(1000));
    }

    // Use RGB.
    let rgb_orange = colour::rgb_to_hsv(colour::RGB {
        red: 255,
        green: 165,
        blue: 0,
    });
    let _ = messages::set_device_state(&device, &rgb_orange, 1000, 0);
    thread::sleep(Duration::from_millis(1000));

    // More constants.
    let cols: Vec<colour::HSB> = vec![
        colour::get_colour("beige"),
        colour::get_colour("chartreuse"), 
        colour::get_colour("coral"), 
        colour::get_colour("cornflower"), 
        colour::get_colour("crimson"), 
        colour::get_colour("deep_sky_blue"), 
        colour::get_colour("slate_gray"), 
    ];

    for c in cols {
        let _ = messages::set_device_state(&device, &c, 1000, 0);
        thread::sleep(Duration::from_millis(1000));
    }

    let device = messages::set_device_state(&device, &colour::get_colour("beige"), 1000, 0);
    display_response("Set state", &device.unwrap().response.unwrap());

}

fn display_response(title: &str, resp: &response::Response) {
    println!("\n{} :", title);
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

    // println!("Service: {:?}", resp.service);
    // println!("Port: {:?}", resp.port);
    // println!("Unknown: {:?}", resp.unknown);

    println!("==========");
}
