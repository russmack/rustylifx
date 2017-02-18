extern crate rustylifx;


fn main() {
    println!("Started.");
    let resp = rustylifx::get_service().unwrap();
    display_response(resp);
    println!("Finished.");
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

    println!("Service: {:?}", resp.service);
    println!("Port: {:?}", resp.port);
    println!("Unknown: {:?}", resp.unknown);
    
}
