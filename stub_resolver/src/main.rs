use shared::DNSRequestID;
use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use std::convert::Infallible;
use tokio;
use futures::future;

type HttpClient = Client<hyper::client::HttpConnector>;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);


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
        // binds port dynamically
        let mut socket = UdpSocket::bind("127.0.0.2:0").await.expect("Failed to bind stub udp socket");

        let mut domain = req.uri().authority().unwrap().as_str().to_owned();
        
        let id = match ask_resolver(&mut socket, &mut domain).await {
            Ok(id) => id,
            Err(e) => {
                println!("ask_resolver failed: {}", e);
                let mut response = Response::new(Body::empty());
                *response.body_mut() = Body::from("failed to resolve");
                return Ok(response);
            }
        };

        // asking successful, wait for answer with correct id

        match wait_for_response(id, &mut socket).await {
            Ok(ip) => {
                println!("IP is {:?}", ip);

                let socket = SocketAddr::new(ip, 8000);

                let uri = hyper::http::Uri::builder()
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

async fn interactive() {
    // create context
    let mut socket = UdpSocket::bind("127.0.0.2:53053").await.expect("Failed to bind stub udp socket");

    loop {
        println!("Enter domain to resolve. enter `exit` to exit.");
        let mut domain = String::new();
        if let Ok(_) = std::io::stdin().read_line(&mut domain) {
            let mut domain = domain.trim().to_string(); // remove newline, not performant but ok for interactive
                                        // exit
            if domain.eq_ignore_ascii_case("exit") {
                break;
            };


            let start_time = std::time::Instant::now();
            println!("Resolving {}...", domain);
            // ask recursive resolver about domain
            let id = match ask_resolver(&mut socket, &mut domain).await {
                Ok(id) => id,
                Err(e) => {
                    println!("ask_resolver failed: {}", e);
                    continue;
                }
            };

            // asking successful, wait for answer with correct id

            match wait_for_response(id, &mut socket).await {
                Ok(ip) => println!("IP is {:?}", ip),
                Err(e) => println!("Error: {:?}", e),
            }
            println!("Query {} took {}s", id.0, start_time.elapsed().as_secs_f32());
        }
    }

    println!("stub terminated");
}


#[tokio::main]
async fn main() {
    let mut handles = Vec::new();
    if std::env::args().any(|s| &s == "proxy") {
        println!("Starting http proxy");
        handles.push(tokio::spawn(http_proxy()));
    }

    if std::env::args().any(|s| &s == "interactive") {
        println!("Starting interactive mode");
        handles.push(tokio::spawn(interactive()));
    }

    // wait till either the proxy or interactive mode terminated
    future::select_ok(handles).await.expect("select_ok failed");
    println!("all finished");
}

/// sends UDP packet to recursive resolver and returns a unique id for that request.
/// Accepts domain with or without root dot '.' at the end, will add if not present
async fn ask_resolver(socket: &mut UdpSocket, domain: &mut String) -> Result<DNSRequestID, &'static str> {
    let id = DNSRequestID(NEXT_ID.fetch_add(1, Ordering::SeqCst));

    if !domain.ends_with(".") {
        domain.push('.');
    }

    // crate dns packet
    let packet = shared::DNSPacket {
        id,
        flags_rec_desired: true,
        flags_response: false,
        flags_result_code: shared::ResultCode::NOERROR,
        flags_authorative: false,
        qry_name: domain.to_string(),
        qry_type: shared::QueryType::A,
        answer_a: None,
        answer_ns: None,
        resp_ttl: None
    };
    // increase id

    // fixed recursive resolver addr: 127.0.0.10:53053
    // never fails if ip address is written correctly
    let rec_addr: IpAddr = shared::RECURSIVE_RESOLVER_ADDR.parse().unwrap();
    let rec_addr: SocketAddr = (rec_addr, shared::PORT).into();
    println!("Sending {} -> {:?}", packet, rec_addr);

    shared::send_dns_packet(socket, &packet, rec_addr).await?;

    // return current id
    Ok(id)
}

/// wait for a response with that id. if the next response is not for that request, returns error
async fn wait_for_response(id: DNSRequestID, socket: &mut UdpSocket) -> Result<IpAddr, &'static str> {
    let (packet, _) = shared::recv_dns_packet(socket).await?;

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
        .ok_or(match packet.flags_result_code {
            shared::ResultCode::NOERROR => "no A record but also NOERROR code, wtf???",
            shared::ResultCode::NXDOMAIN => "NXDOMAIN: no ip exists for this domain",
            _ => "unhandled result code in packet, no A record present"
        })
}
