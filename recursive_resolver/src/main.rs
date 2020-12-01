use std::net::{Ipv4Addr, UdpSocket};

fn main() {
    println!("Hello from Recursive");


    // bind to 127.0.0.10:53053
    let socket = UdpSocket::bind(shared::RECURSIVE_RESOLVER_ADDR).expect("Failed to bind recursive to fixed addr");


    loop {
        // receive packet

        let (req_packet, req_sender) = match shared::recv_dns_packet(&socket) {
            Err(e) => {println!("Error while receiving packet: {}", e); continue },
            Ok(r) => r
        };

        println!("Received {:?} from {:?}", req_packet, req_sender);

        // look at cache

        // iterative solve by asking DNS servers

        // return answer

        // !!! this is only for testing udp response
        let answer_packet = shared::DNSPacket {
            flags_response: true,
            answer_a: Some(Ipv4Addr::from([1, 2, 3, 4])), // test addr
            ..req_packet
        };
        match shared::send_dns_packet(&socket, answer_packet, req_sender) {
            Ok(()) => println!("response send"),
            Err(e) => println!("error: {}", e),
        }
    }
}
