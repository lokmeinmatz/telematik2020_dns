use std::net::UdpSocket;

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
    }

}
