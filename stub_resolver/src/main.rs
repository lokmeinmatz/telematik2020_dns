use std::{net::IpAddr, unimplemented};

use shared::DNSRequestID;

fn main() {
    println!("Hello from Stub");
    loop {
        println!("Enter domain to resolve. enter `exit` to exit.");
        
        let mut domain = String::new();
        
        if let Ok(_) = std::io::stdin().read_line(&mut domain) {
            
            let domain = domain.trim(); // remove newline
            // exit
            if domain.eq_ignore_ascii_case("exit") { break };

            println!("Resolving {}...", domain);
            
            
            // ask recursive resolver about domain
            let id = ask_resolver(&domain);

            match wait_for_response(id) {
                Ok(ip) => println!("IP is {:?}", ip),
                Err(e) => println!("Error: {:?}", e)
            }
            
        }
    }

    println!("stub terminated");
}

/// sends UDP packet to recursive resolver and returns a unique id for that request.
fn ask_resolver(domain: &str) -> DNSRequestID {
    unimplemented!()
}


/// wait for a response with that id. if the next response is not for that request, returns error
fn wait_for_response(id: DNSRequestID) -> Result<IpAddr, &'static str> {
    unimplemented!()
}