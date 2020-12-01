use std::net::{Ipv4Addr, UdpSocket, SocketAddr, IpAddr};
use shared::ResultCode;

fn main() {
    println!("Hello from Recursive");

    // bind to 127.0.0.10:53053
    let socket_in = UdpSocket::bind((shared::RECURSIVE_RESOLVER_ADDR, shared::PORT)).expect("Failed to bind recursive to fixed addr in");
    // bind to 127.0.0.10:53054
    let socket_out = UdpSocket::bind((shared::RECURSIVE_RESOLVER_ADDR, 53054)).expect("Failed to bind recursive to fixed addr out");


    loop {
        // receive packet

        let (req_packet, req_sender) = match shared::recv_dns_packet(&socket_in) {
            Err(e) => {println!("Error while receiving packet: {}", e); continue },
            Ok(r) => r
        };

        println!("Received {:?} from {:?}", req_packet, req_sender);

        // Cleanup and splitting
        let mut domain_name: Vec<&str> = req_packet.qry_name.trim().split(".").collect();

        // virtually add last dot if not provided
        match domain_name.last() {
            Some(&"") => (),
            None => {println!("Error parsing domain name, address must not be empty"); continue}
            _ => domain_name.push(""),
        }
        
        // look at cache

        // iterative solve by asking DNS servers
        let mut ns: Ipv4Addr = shared::ROOT_SERVER_ADDR.parse().unwrap();

        let mut packet = req_packet;

        packet.flags_rec_desired = false;

        let id = packet.id;

        loop{

            let rec_addr = SocketAddr::new(IpAddr::V4(ns), shared::PORT);

            println!("Sending {:?} -> {:?}", packet, rec_addr);

            shared::send_dns_packet(&socket_out, &packet, rec_addr).unwrap();

            let (recv_packet, _) = shared::recv_dns_packet(&socket_out).unwrap();

            if recv_packet.id != id {
                println!("response was not for the request we send last");
                continue;
            }

          
            match (recv_packet.answer_ns, recv_packet.answer_a, recv_packet.flags_result_code) {
                (Some(ns_rec), _, ResultCode::NOERROR) => ns = ns_rec,
                _ => {
                    println!("received non-recursive answer");
                    match shared::send_dns_packet(&socket_in, &recv_packet, req_sender) {
                        Ok(()) => println!("response send"),
                        Err(e) => println!("error: {}", e),
                    }
                    break;
                }
            }
            
        }

    }
}
