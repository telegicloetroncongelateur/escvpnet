use crate::{
    Command, Error, Header, Identifier, Packet, ReadPacket, ReceivePacket, Response, SendCommand,
    Status, WritePacket,
};
use std::{
    net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
    time::Duration,
    thread::sleep,
};

type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    stream: TcpStream,
}

pub const HELLO: [u8; 16] = *b"ESC/VP.net\x10\x01\x00\x00\x00\x00";

impl Client {
    pub fn connect<A: ToSocketAddrs>(addr: A, password: Option<String>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr)?;

        let headers = password.map(|password| vec![Header::Password(Some(password))]);

        let packet = Packet::new(Identifier::Connect, Status::Null, headers);

        stream.write_packet(packet)?;
        stream.read_packet()?.status.as_result()?;

        Ok(Self { stream })
    }

    pub fn discover<A: ToSocketAddrs>(
        addr: A,
        broadcast: A,
        timeout: Option<Duration>,
    ) -> Result<Vec<(SocketAddr, Option<String>)>> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_broadcast(true)?;
        socket.set_read_timeout(timeout)?;
        socket.set_write_timeout(timeout)?;

        socket.send_to(&HELLO, broadcast)?;
        let mut projectors = Vec::new();
        
        if let Some(timeout) = timeout {

            sleep(timeout);
        } 
        'a: while let Ok((packet, addr)) = socket.recv_packet_from() {
            if packet.status == Status::Null {
                continue;
            }
            if let Some(headers) = packet.headers {
                for header in headers {
                    if let Header::ProjectorName(name) = header {
                        projectors.push((addr, name));
                        continue 'a;
                    }
                }
            }
            projectors.push((addr, None));
        }
        Ok(projectors)
    }

    pub fn send(&mut self, command: Command) -> Result<Response> {
        self.stream.send_command(command)
    }
}
