use std::net::{Ipv4Addr, UdpSocket};
use serde_json;
use shared::DNSPacket;
use std::io::Read;
use std::str::FromStr;
use serde::Deserialize;

#[derive(Debug)]
struct ServerConfig {
    zone: Vec<(String, Ipv4Addr)>,
    delegated: Vec<(String, Ipv4Addr)>,
    ip: Ipv4Addr
}

fn main() {

    

    let config_name = std::env::args().nth(1).expect("dns_server <server_name>.json to start dns server");
    
    if !config_name.ends_with(".json") {
        panic!("config needs json format");
    }

    let server_name = &config_name[..config_name.len() - 5];
    println!("Hello from DNS Server {}", server_name);

    // load server config from server_configs/<path>
    let server_config: ServerConfig = {
        let mut config_file = std::fs::File::open(format!("./server_configs/{}", config_name)).expect("Config file not found :(");
        let mut data = String::new();
        config_file.read_to_string(&mut data).expect("read failed");

        #[derive(Deserialize)]
        struct JsonConfig {
            ip: String,
            zone: serde_json::Map<String, serde_json::Value>,
            delegated: serde_json::Map<String, serde_json::Value>
        };

        let json_config: JsonConfig = serde_json::from_str(&data).expect("Failed to parse json data, must be object with schema");

        let to_serverconfig = |(domain, value)| {
            match value {
                serde_json::Value::String(ip) => Ipv4Addr::from_str(&ip).ok().map(|ip| (domain, ip)),
                _ => {
                    println!("config not correct formatted, ip must be a string");
                    None
                }
            }
        };

        ServerConfig {
            zone: json_config.zone.into_iter().filter_map(to_serverconfig).collect(),
            delegated: json_config.delegated.into_iter().filter_map(to_serverconfig).collect(),
            ip: Ipv4Addr::from_str(&json_config.ip).unwrap()
        }
    };

    println!("Zone config: {:?}", server_config);

    // bind to 127.0.0.100:53053
    let socket_in = UdpSocket::bind((server_config.ip, shared::PORT)).expect("Failed to bind recursive to fixed addr in");


    loop {
        // receive packet

        let (req_packet, req_sender) = match shared::recv_dns_packet(&socket_in) {
            Err(e) => {println!("Error while receiving packet: {}", e); continue },
            Ok(r) => r
        };

        println!("Received {:?} from {:?}", req_packet, req_sender);

        let mut answer_packet = DNSPacket {
            flags_response: true,
            answer_a: None,
            answer_ns: None,
            ..req_packet
        };

        
        // check records

        for (domain, ip) in &server_config.zone {
            if &answer_packet.qry_name == domain {
                answer_packet.answer_a = Some(*ip);
                break;
            }
        }
        if answer_packet.answer_a.is_none() {
            for (zone, ip) in &server_config.delegated {
                if answer_packet.qry_name.ends_with(zone){
                    answer_packet.answer_ns = Some(*ip)
                }
            }
        }
        
        println!("sending response {:?}", answer_packet);
        shared::send_dns_packet(&socket_in, &answer_packet, req_sender).unwrap();
    }

}
