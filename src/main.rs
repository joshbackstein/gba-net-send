//#![allow(dead_code, unused)]
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket};
use std::fs::File;
use std::{env, thread, time};

const APP_TITLE: &str = "GBA Net Send v0.0.1 Beta";
const APP_NAME: &str = "gba-net-send";
const RETRIES: u32 = 10;
const PORT: u16 = 31313;
const INIT_REQUEST: &str = "gba_net_boot_init_beta_0001";
const INIT_RESPONSE: &str = "gba_net_boot_ack_beta_0001";

fn main() {
    println!("{}", APP_TITLE);

    let mut args = env::args();
    if args.len() == 1 {
        println!("Usage: {} <rom-path>", APP_NAME);
        return;
    }

    // Check to make sure ROM path was provided
    let rom_path = match args.nth(1) {
        Some(p) => p,
        None => {
            eprintln!("Could not retrieve argument for ROM path");
            return;
        }
    };

    // Get UDP socket so we can broadcast to gba-net-boot loaders
    let udp = match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT)) {
        Ok(udp) => {
            println!("Bound to UDP port {}", PORT);
            udp
        }
        Err(err) => {
            eprintln!("Could not bind UDP socket");
            eprintln!("{}", err);
            return;
        }
    };
    println!("Setting UDP socket to broadcast");
    if let Err(err) = udp.set_broadcast(true) {
        eprintln!("Could not set up UDP socket to broadcast");
        eprintln!("{}", err);
        return;
    }

    // Broadcast and check for responses
    println!("Broadcasting to UDP socket");
    let mut tcp_addr: Option<SocketAddr> = None;
    for _ in 0..RETRIES {
        if let Err(err) = udp.send_to(INIT_REQUEST.as_bytes(), SocketAddrV4::new(Ipv4Addr::BROADCAST, PORT))
        {
            eprintln!("Could not broadcast to UDP socket");
            eprintln!("{}", err);
            return;
        }

        // Wait a bit for a response
        thread::sleep(time::Duration::from_millis(100));

        // Check for response
        let mut udp_buf = [0; 0x1000];
        let (num_recvd_bytes, src_addr) = match udp.recv_from(&mut udp_buf) {
            Ok((n, s)) => (n, s),
            Err(err) => {
                eprintln!("Could not receive bytes from UDP socket");
                eprintln!("{}", err);
                return;
            }
        };
        let filled_udp_buf = &mut udp_buf[..num_recvd_bytes];
        let filled_vec = filled_udp_buf.to_vec();
        let udp_buf_string = match String::from_utf8(filled_vec) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Could not convert byte array to string");
                eprintln!("{}", err);
                return;
            }
        };
        if udp_buf_string == INIT_RESPONSE {
            println!("Received ack from IP {}", src_addr.ip());
            tcp_addr = Some(src_addr);
            break;
        }
    }
    let tcp_addr = match tcp_addr {
        Some(a) => a,
        None => {
            println!("No response received from 3DS");
            return;
        }
    };

    // Open ROM file
    // This needs to happen AFTER the ack to make sure we're sending the most recent copy of the file
    // in case we've been waiting a while for an ack
    let mut file = match File::open(&rom_path) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Could not open ROM: {}", rom_path);
            eprintln!("{}", err);
            return;
        }
    };

    // Establish TCP connection to broadcast responder
    let mut tcp_stream = match TcpStream::connect(format!("{}:{}", tcp_addr.ip(), PORT)) {
        Ok(tcp_stream) => {
            println!("Connection established to {}:{}", tcp_addr.ip(), PORT);
            tcp_stream
        }
        Err(err) => {
            eprintln!("Could not establish connection to {}:{}", tcp_addr.ip(), PORT);
            eprintln!("{}", err);
            return;
        }
    };

    // Loop to read file into buffer and send over TCP
    let mut tcp_buf = [0; 4096];
    let mut total_bytes_written = 0;
    loop {
        // Read to buffer
        let num_read = match file.read(&mut tcp_buf) {
            Ok(n) => n,
            Err(err) => {
                eprintln!("Could not read from ROM file");
                eprintln!("{}", err);
                return;
            }
        };

        if num_read == 0 {
            // If we can't read anything else, we've reached the end of the file
            break;
        }

        let num_written = match tcp_stream.write(&tcp_buf) {
            Ok(n) => n,
            Err(err) => {
                eprintln!("Could not write to TCP stream");
                eprintln!("{}", err);
                return;
            }
        };
        print!("\rSent {} bytes to {}", total_bytes_written, tcp_addr.ip());

        total_bytes_written += num_written;
    }
    println!("\rSent {} bytes to {}", total_bytes_written, tcp_addr.ip());
}
