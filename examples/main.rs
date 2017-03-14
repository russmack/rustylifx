extern crate rustylifx;

use rustylifx::{colour, request, response};

use std::time::Duration;
use std::thread;

fn main() {
    // Find device.
    let device = request::get_service().unwrap();
    display_response("State service", &device.response);
    thread::sleep(Duration::from_millis(1000));

    // Get device state.
    let device = request::get_device_state(device).unwrap();
    display_response("State", &device.response);
    thread::sleep(Duration::from_millis(1000));

    // Parse out HSVK details.
    println!("\nCurrent state received:");
    let payload = match device.response.payload {
        response::Payload::State(ref v) => Some(v),
        _ => None,
    };

    match payload {
        Some(v) => {
            println!("current payload body: {:?}", v.body);
            println!("current hue: {:?}", v.hue);
            println!("current sat: {:?}", v.saturation);
            println!("current bri: {:?}", v.brightness);
            println!("current kel: {:?}", v.kelvin);
        },
        None => (),
    };
    println!("\n");


    // Set colour.

    // Use constants.
    let cols: Vec<colour::HSB> = vec![colour::RED, colour::GREEN, colour::BLUE];

    for c in cols {
        let _ = request::set_device_state(&device, c, 1000, 0);
        thread::sleep(Duration::from_millis(1000));
    }

    // Use RGB.
    let rgb_orange = colour::rgb_to_hsv(colour::RGB {
        red: 255,
        green: 165,
        blue: 0,
    });
    let _ = request::set_device_state(&device, rgb_orange, 1000, 0);
    thread::sleep(Duration::from_millis(1000));

    // More constants.
    let cols: Vec<colour::HSB> = vec![
        colour::BEIGE, 
        colour::CHARTREUSE, 
        colour::CORAL, 
        colour::CORNFLOWER, 
        colour::CRIMSON, 
        colour::DEEP_SKY_BLUE, 
        colour::SLATE_GRAY, 
    ];

    for c in cols {
        let _ = request::set_device_state(&device, c, 1000, 0);
        thread::sleep(Duration::from_millis(1000));
    }

    let device = request::set_device_state(&device, colour::BEIGE, 1000, 0);
    display_response("Set state", &device.unwrap().response);

    println!("\nFinished.");
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
