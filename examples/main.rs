extern crate rustylifx;

use rustylifx::colour;

use std::time::Duration;
use std::thread;

fn main() {
    println!("Started.");
    let resp = rustylifx::request::get_service().unwrap();
    println!("\nState service: {:?}", resp);
    println!("==========\n");
    //display_response(resp);

    thread::sleep(Duration::from_millis(1000));
    let resp2 = rustylifx::request::get_device_state().unwrap();
    println!("\nState: {:?}", resp2);
    println!("==========");
    //display_response(resp2);

    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::RED, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::GREEN, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::BLUE, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::BEIGE, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::CHARTREUSE, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::CORAL, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::CORNFLOWER, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::DEEP_SKY_BLUE, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::SLATE_GRAY, 65000, 65000, 1000, 0);
    thread::sleep(Duration::from_millis(1000));
    rustylifx::request::set_device_state(colour::BEIGE, 65000, 65000, 1000, 0);

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
