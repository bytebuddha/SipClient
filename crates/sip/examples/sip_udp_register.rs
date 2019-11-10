//! This example was created using a virtualbox vm
//! running ubuntu 19.04 and fusionpbx running at
//! 192.168.1.123
//!
//! This expects and extension with the numeric id `20`
//! and a password of `program`

extern crate nirah_sip;
#[macro_use]
extern crate nirah_uri;

use nirah_uri::Uri;
use nirah_sip::*;
use nirah_uri::Param;
use nirah_uri::Transport;
use nirah_uri::parse_uri;
use nirah_sip::core::message::parse_response;
use nirah_sip::client::RegistrationManager;

use std::io;
use std::net::UdpSocket;

fn get_our_uri() -> Uri {
    Uri::sip(ip_domain!(192, 168, 1, 76, 5060))
        .auth(uri_auth!("20"))
        .parameter(Param::Transport(Transport::Udp))
}

fn send_request_get_response(req: SipMessage) -> Result<SipMessage, io::Error> {
    let addr = "0.0.0.0:5060";
    let sock = UdpSocket::bind(addr)?;
    sock.send_to(&format!("{}", req).as_ref(), "192.168.1.120:5060")?;
    let mut buf = vec![0; 65535];
    let (amt, _src) = sock.recv_from(&mut buf)?;
    if let Err(nom::Err::Error((data, _))) = parse_response(&buf[..amt]) {
        panic!("{}", String::from_utf8_lossy(data));
    }
    let (_, msg) = parse_response(&buf[..amt]).unwrap();
    Ok(msg)
}


fn main() -> Result<(), io::Error>{
    let acc_url = parse_uri(b"sip:20@192.168.1.120 ").unwrap().1
            .parameter(Param::Transport(Transport::Udp));
    let mut builder = RegistrationManager::new(acc_url, get_our_uri(), Default::default());
    builder.username("20");
    builder.password("program");

    let req = builder.get_request()?;
    println!("{}", req);

    let res = send_request_get_response(req)?;
    println!("{}\n", &res);
    builder.set_challenge(res)?;

    let authed = builder.get_request()?;
    println!("\n{}\n", authed);

    let res = send_request_get_response(authed)?;
    println!("{}\n", res);
    Ok(())
}
