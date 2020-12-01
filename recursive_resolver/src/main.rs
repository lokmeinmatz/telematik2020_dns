use std::net::{Ipv4Addr, UdpSocket, SocketAddr, IpAddr};

fn main() {
    println!("Hello from Recursive");


    // bind to 127.0.0.10:53053
    let socket_in = UdpSocket::bind(shared::RECURSIVE_RESOLVER_ADDR).expect("Failed to bind recursive to fixed addr in");
    // bind to 127.0.0.10:53054
    let socket_out = UdpSocket::bind("127.0.0.10:53054").expect("Failed to bind recursive to fixed addr out");


    loop {
        // receive packet

        let (mut req_packet, req_sender) = match shared::recv_dns_packet(&socket_in) {
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
        let mut ns: Ipv4Addr = Ipv4Addr::from([127, 0, 0, 100]);

        loop{

            let mut packet = req_packet;

            let id = packet.id;
            
            packet.flags_rec_desired = false;

            let rec_addr = SocketAddr::new(IpAddr::V4(ns), 53053);

            println!("Sending {:?} -> {:?}", packet, rec_addr);

            shared::send_dns_packet(&socket_out, packet, rec_addr).unwrap();

            let (recv_packet, _) = shared::recv_dns_packet(&socket_out).unwrap();

            if recv_packet.id != id {
                println!("response was not for the request we send last");
                continue;
            }

            break;

            //TODO: afaik we need new fields in our dns packet to do recursive resolving

            // if valid response, break;

            // else: ns = new_ns;

        }

        // return answer

        // !!! this is only for testing udp response
        let answer_packet = shared::DNSPacket {
            flags_response: true,
            answer_a: Some(Ipv4Addr::from([1, 2, 3, 4])), // test addr
            ..req_packet
        };
        match shared::send_dns_packet(&socket_in, answer_packet, req_sender) {
            Ok(()) => println!("response send"),
            Err(e) => println!("error: {}", e),
        }
    }
}
