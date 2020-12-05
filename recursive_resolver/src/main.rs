use std::{collections::HashMap, net::{Ipv4Addr, SocketAddr, IpAddr}};
use tokio::net::UdpSocket;
use shared::ResultCode;

type TTLEnd = std::time::SystemTime;

#[tokio::main]
async fn main() {
    println!("Hello from Recursive");

    // bind to 127.0.0.10:53053
    let mut socket_in = UdpSocket::bind((shared::RECURSIVE_RESOLVER_ADDR, shared::PORT)).await.expect("Failed to bind recursive to fixed addr in");
    // bind to 127.0.0.10:53054
    let mut socket_out = UdpSocket::bind((shared::RECURSIVE_RESOLVER_ADDR, 53054)).await.expect("Failed to bind recursive to fixed addr out");

    let mut cache: HashMap<String, (Ipv4Addr, TTLEnd)> = HashMap::new();

    loop {
        // receive packet

        let (req_packet, req_sender) = match shared::recv_dns_packet(&mut socket_in).await {
            Err(e) => {println!("Error while receiving packet: {}", e); continue },
            Ok(r) => r
        };

        println!("Received {} from {:?}", req_packet, req_sender);

        
        
        // look at cache

        if let Some((ip, valid_until)) = cache.get(&req_packet.qry_name) {
            
            if &std::time::SystemTime::now() < valid_until {
                let mut answer_packet = req_packet;
                // send cached ip
                answer_packet.flags_response = true;
                answer_packet.flags_result_code = ResultCode::NOERROR;
                answer_packet.answer_a = Some(*ip);
                println!("using cached ip");
                if let Err(e) = shared::send_dns_packet(&mut socket_in, &answer_packet, req_sender).await {
                    println!("Error while sending cached response: {}", e);
                }
                continue;
            } else {
                println!("Found in cache but ttl expired");
            }
        }

        // iterative solve by asking DNS servers
        let mut ns: Ipv4Addr = shared::ROOT_SERVER_ADDR.parse().unwrap();

        let mut packet = req_packet;

        packet.flags_rec_desired = false;

        let id = packet.id;

        loop{

            let rec_addr = SocketAddr::new(IpAddr::V4(ns), shared::PORT);

            println!("Sending {} -> {:?}", packet, rec_addr);

            shared::send_dns_packet(&mut socket_out, &packet, rec_addr).await.unwrap();

            let (recv_packet, _) = shared::recv_dns_packet(&mut socket_out).await.unwrap();

            if recv_packet.id != id {
                println!("response was not for the request we send last");
                continue;
            }

          
            match (recv_packet.answer_ns, recv_packet.answer_a, recv_packet.flags_result_code) {
                (Some(ns_rec), _, ResultCode::NOERROR) => ns = ns_rec,
                (None, Some(a_rec), ResultCode::NOERROR) => {
                    if let Some(ttl) = recv_packet.resp_ttl {
                        // add to cache
                        cache.insert(recv_packet.qry_name.clone(), (a_rec, std::time::SystemTime::now() + ttl));
                        println!("cache entry updated");
                    }

                    match shared::send_dns_packet(&mut socket_in, &recv_packet, req_sender).await {
                        Ok(()) => println!("response send"),
                        Err(e) => println!("error: {}", e),
                    }
                    break;
                }
                _ => {
                    match shared::send_dns_packet(&mut socket_in, &recv_packet, req_sender).await {
                        Ok(()) => println!("response send"),
                        Err(e) => println!("error: {}", e),
                    }
                    break;
                }
            }
            
        }

    }
}
