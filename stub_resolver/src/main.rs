use shared::DNSRequestID;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server};
use std::convert::Infallible;
use tokio;

type HttpClient = Client<hyper::client::HttpConnector>;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn http_proxy() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8100));
    let client = HttpClient::new();

    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| proxy(client.clone(), req))) }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn proxy(client: HttpClient, mut req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    println!("req: {:?}", req.uri().to_string());

    if req.uri().host().is_some()
        && (req.uri().host().unwrap().contains(".telematik")
            || req.uri().host().unwrap().contains(".fuberlin"))
    {
        println!("Resolving {}...", req.uri().authority().unwrap().as_str());
        // ask recursive resolver about domain
        // create context
        let socket = UdpSocket::bind("127.0.0.2:53053").expect("Failed to bind stub udp socket");

        let mut domain = req.uri().authority().unwrap().as_str().to_owned();
        if !domain.ends_with(".") {
            domain.push('.');
        }
        let id = match ask_resolver(&socket, &domain) {
            Ok(id) => id,
            Err(e) => {
                println!("ask_resolver failed: {}", e);
                let mut response = Response::new(Body::empty());
                *response.body_mut() = Body::from("failed to resolve");
                return Ok(response);
            }
        };

        // asking successful, wait for answer with correct id

        match wait_for_response(id, &socket) {
            Ok(ip) => {
                println!("IP is {:?}", ip);

                let socket = SocketAddr::new(ip, 8000);

                let mut uri = hyper::http::Uri::builder()
                    .scheme(req.uri().scheme_str().unwrap())
                    .authority(socket.to_string().as_str())
                    .path_and_query(req.uri().path_and_query().unwrap().as_str())
                    .build()
                    .unwrap();
                *req.uri_mut() = uri;
                client.request(req).await
            }
            Err(e) => {
                println!("Error: {:?}", e);
                let mut response = Response::new(Body::empty());
                *response.body_mut() = Body::from("failed to resolve");
                return Ok(response);
            }
        }
    } else {
        client.request(req).await
    }
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "http" {
        http_proxy();
    }

    println!("Hello from Stub");

    // create context
    let socket = UdpSocket::bind("127.0.0.2:53053").expect("Failed to bind stub udp socket");

    loop {
        println!("Enter domain to resolve. enter `exit` to exit.");
        let mut domain = String::new();
        if let Ok(_) = std::io::stdin().read_line(&mut domain) {
            let domain = domain.trim(); // remove newline
                                        // exit
            if domain.eq_ignore_ascii_case("exit") {
                break;
            };

            println!("Resolving {}...", domain);
            // ask recursive resolver about domain
            let id = match ask_resolver(&socket, &domain) {
                Ok(id) => id,
                Err(e) => {
                    println!("ask_resolver failed: {}", e);
                    continue;
                }
            };

            // asking successful, wait for answer with correct id

            match wait_for_response(id, &socket) {
                Ok(ip) => println!("IP is {:?}", ip),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }

    println!("stub terminated");
}

/// sends UDP packet to recursive resolver and returns a unique id for that request.
fn ask_resolver(socket: &UdpSocket, domain: &str) -> Result<DNSRequestID, &'static str> {
    let id = DNSRequestID(NEXT_ID.fetch_add(1, Ordering::SeqCst));
    // crate dns packet
    let packet = shared::DNSPacket {
        id,
        flags_rec_desired: true,
        flags_response: false,
        qry_name: domain.to_string(),
        qry_type: shared::QueryType::A,
        answer_a: None,
        flags_result_code: shared::ResultCode::NOERROR,
        answer_ns: None,
    };
    // increase id

    // fixed recursive resolver addr: 127.0.0.10:53053
    // never fails if ip address is written correctly
    let rec_addr: IpAddr = shared::RECURSIVE_RESOLVER_ADDR.parse().unwrap();
    let rec_addr: SocketAddr = (rec_addr, shared::PORT).into();
    println!("Sending {:?} -> {:?}", packet, rec_addr);

    shared::send_dns_packet(&socket, &packet, rec_addr)?;

    // return current id
    Ok(id)
}

/// wait for a response with that id. if the next response is not for that request, returns error
fn wait_for_response(id: DNSRequestID, socket: &UdpSocket) -> Result<IpAddr, &'static str> {
    let (packet, _) = shared::recv_dns_packet(&socket)?;

    if !packet.flags_response {
        return Err("received packet was not a response :(");
    }
    if packet.id != id {
        return Err("response was not for the request we send last");
    }

    // convert the ipv4 address to a genereic ip address or fail if no A record was provided
    packet
        .answer_a
        .map(|addr| IpAddr::V4(addr))
        .ok_or("no ip address in answer")
}
