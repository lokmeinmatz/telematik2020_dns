use std::net::{IpAddr, SocketAddr, UdpSocket};

use shared::DNSRequestID;





struct Context {
    socket: UdpSocket,
    next_packet_id: DNSRequestID
}

fn main() {
    println!("Hello from Stub");

    // create context
    let mut ctx = Context {
        socket: UdpSocket::bind("127.0.0.2:53053").expect("Failed to bind stub udp socket"),
        next_packet_id: DNSRequestID(1)
    };


    loop {
        println!("Enter domain to resolve. enter `exit` to exit.");
        
        let mut domain = String::new();
        
        if let Ok(_) = std::io::stdin().read_line(&mut domain) {
            
            let domain = domain.trim(); // remove newline
            // exit
            if domain.eq_ignore_ascii_case("exit") { break };

            println!("Resolving {}...", domain);
            
            
            // ask recursive resolver about domain
            let id = match ask_resolver(&mut ctx, &domain) {
                Ok(id) => id,
                Err(e) => {
                    println!("ask_resolver failed: {}", e);
                    continue;
                }
            };

            // asking successful, wait for answer with correct id

            match wait_for_response(id, &ctx) {
                Ok(ip) => println!("IP is {:?}", ip),
                Err(e) => println!("Error: {:?}", e)
            }
            
        }
    }

    println!("stub terminated");
}

/// sends UDP packet to recursive resolver and returns a unique id for that request.
fn ask_resolver(ctx: &mut Context, domain: &str) -> Result<DNSRequestID, &'static str> {
    let id = ctx.next_packet_id;
    // crate dns packet
    let packet = shared::DNSPacket {
        id,
        flags_rec_desired: true,
        flags_response: false,
        qry_name: domain.to_string(),
        qry_type: shared::QueryType::A,
        answer_a: None,
        flags_result_code: shared::ResultCode::NOERROR,
        answer_ns: None
    };
    // increase id
    ctx.next_packet_id.0 += 1;

    // fixed recursive resolver addr: 127.0.0.10:53053
    // never fails if ip address is written correctly
    let rec_addr : IpAddr = shared::RECURSIVE_RESOLVER_ADDR.parse().unwrap();
    let rec_addr : SocketAddr = (rec_addr, shared::PORT).into();
    println!("Sending {:?} -> {:?}", packet, rec_addr);

    shared::send_dns_packet(&ctx.socket, &packet, rec_addr)?;

    // return current id
    Ok(id)
}


/// wait for a response with that id. if the next response is not for that request, returns error
fn wait_for_response(id: DNSRequestID, ctx: &Context) -> Result<IpAddr, &'static str> {
    let (packet, _) = shared::recv_dns_packet(&ctx.socket)?;

    if !packet.flags_response {
        return Err("received packet was not a response :(");
    }
    if packet.id != id {
        return Err("response was not for the request we send last");
    }

    // convert the ipv4 address to a genereic ip address or fail if no A record was provided
    packet.answer_a.map(|addr| IpAddr::V4(addr)).ok_or("no ip address in answer")
}
