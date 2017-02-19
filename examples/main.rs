extern crate rustylifx;

use std::time::Duration;
use std::thread;

fn main() {
    println!("Started.");
    let resp = rustylifx::get_service().unwrap();
    println!("\nState service: {:?}", resp);
    println!("==========\n");
    //display_response(resp);

    thread::sleep(Duration::from_millis(1000));
    let resp2 = rustylifx::get_device_state().unwrap();
    println!("\nState: {:?}", resp2);
    println!("==========");
    //display_response(resp2);
    println!("\nFinished.");
}

fn display_response(resp: rustylifx::Response) {
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
