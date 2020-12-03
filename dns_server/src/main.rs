use std::net::{Ipv4Addr, UdpSocket};

use shared::DNSPacket;

fn main() {
    println!("Hello from DNS Server");

    // bind to 127.0.0.100:53053
    let socket_in = UdpSocket::bind((shared::ROOT_SERVER_ADDR, shared::PORT)).expect("Failed to bind recursive to fixed addr in");

    loop {
        // receive packet

        let (req_packet, req_sender) = match shared::recv_dns_packet(&socket_in) {
            Err(e) => {println!("Error while receiving packet: {}", e); continue },
            Ok(r) => r
        };

        println!("Received {:?} from {:?}", req_packet, req_sender);

        // test send dns answer

        let answer_packet = DNSPacket {
            flags_response: true,
            answer_a: Some([1, 2, 3, 4].into()),
            ..req_packet
        };
        println!("sending response {:?}", answer_packet);
        shared::send_dns_packet(&socket_in, &answer_packet, req_sender).unwrap();
    }

}
