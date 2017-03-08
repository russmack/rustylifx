extern crate rustylifx;

use rustylifx::colour;

use std::time::Duration;
use std::thread;

fn main() {
    // Find device.
    let resp = rustylifx::request::get_service().unwrap();
    println!("\nState service:");
    display_response(resp);
    println!("==========\n");
    thread::sleep(Duration::from_millis(1000));

    // Get device state.
    let resp2 = rustylifx::request::get_device_state().unwrap();
    println!("\nState:");
    display_response(resp2);
    println!("==========");
    thread::sleep(Duration::from_millis(1000));

    // Set colour.

    // Use constants.
    let cols: Vec<colour::HSB> = vec![colour::RED, colour::GREEN, colour::BLUE];

    for c in cols {
        let _ = rustylifx::request::set_device_state(c, 1000, 0);
        thread::sleep(Duration::from_millis(1000));
    }

    // Use RGB.
    let rgb_orange = colour::rgb_to_hsv(colour::RGB{red:255, green:165, blue:0}); 
    let _ = rustylifx::request::set_device_state(rgb_orange, 1000, 0);
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
        let _ = rustylifx::request::set_device_state(c, 1000, 0);
        thread::sleep(Duration::from_millis(1000));
    }

    let resp = rustylifx::request::set_device_state(colour::BEIGE, 1000, 0);
    display_response(resp.unwrap());

    println!("\nFinished.");
}

fn display_response(resp: rustylifx::response::Response) {
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

    //println!("Service: {:?}", resp.service);
    //println!("Port: {:?}", resp.port);
    //println!("Unknown: {:?}", resp.unknown);
}
