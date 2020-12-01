use std::{time::Duration, net::{Ipv4Addr, SocketAddr, UdpSocket}};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct DNSRequestID(pub u32);

#[derive(Serialize, Deserialize, Debug)]
pub enum QueryType {
    A
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DNSPacket {

    pub id: DNSRequestID,
    
    #[serde(rename = "dns.flags.response")]
    pub flags_response: bool,
    
    #[serde(rename = "dns.flags.recdesired")]
    pub flags_rec_desired: bool,
    
    #[serde(rename = "dns.flags.rcode")]
    pub flags_result_code: ResultCode,

    #[serde(rename = "dns.qry.name")]
    pub qry_name: String,

    #[serde(rename = "dns.qry.type")]
    pub qry_type: QueryType,

    #[serde(rename = "dns.a")]
    pub answer_a: Option<Ipv4Addr>,

    #[serde(rename = "dns.ns")]
    pub answer_ns: Option<Ipv4Addr>
}


/// tries to send a udp packet containing the json data to the receiver
pub fn send_dns_packet(socket: &UdpSocket, packet: &DNSPacket, receiver: SocketAddr) -> Result<(), &'static str> {
    // serialize packet
    let bytes = serde_json::to_vec(packet).map_err(|_| "failed to serialize packet")?;

    // wait 100ms 
    std::thread::sleep(Duration::from_millis(100));

    // send bytes
    socket.send_to(&bytes, receiver).map(drop).map_err(|_| "Failed to send over udp")
}


/// interprets the next udp packet as our `DNSPacket` and returns it together with the address of the sender
pub fn recv_dns_packet(socket: &UdpSocket) -> Result<(DNSPacket, SocketAddr), &'static str> {
    // creates 32kb buffer, is this enough?
    let mut buffer = Box::new([0u8; 1024 * 32]);

    // receive udp packet into buffer
    socket.recv_from(&mut buffer[..]).and_then(|(bytes_read, addr)| {

        // deserialize from byte array with length of bytes received
        let packet = serde_json::from_slice(&buffer[0..bytes_read])?;
        Ok((packet, addr))

    }).map_err(|_| "recv_from failed")
}


pub const RECURSIVE_RESOLVER_ADDR: &str = "127.0.0.10";
pub const ROOT_SERVER_ADDR: &str = "127.0.0.100";
pub const PORT: u16 = 53053;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
