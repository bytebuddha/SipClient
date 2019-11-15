use tokio::net::UdpSocket;
use nirah_sip::client::SoftPhone;
use nirah_sip::client::InviteHelper;
use nirah_uri::Uri;
use nirah_uri::UriAuth;
use nirah_uri::Param;
use nirah_uri::Transport;
use nirah_uri::parse_domain;

use crate::prelude::*;
use crate::config::keys::default_ip_interface;

use std::io;
use std::time::Instant;

macro_rules! unwrap_or_else_not_connected {
    ($self:tt, $field: tt, $msg: tt) => {
        if let Some(key) = &$self.$field {
            key
        } else {
            return crate::session::sip::not_connected($msg);
        }
    }
}

macro_rules! unwrap_mut_or_else_not_connected {
    ($self:tt, $field: tt, $msg: tt) => {
        if let Some(key) = &mut $self.$field {
            key
        } else {
            return crate::session::sip::not_connected($msg);
        }
    }
}

mod cancel;
mod invite;
mod message;
mod register;
mod session;

pub struct SipSessionProvider {
    acc: Option<Account>,
    address: Option<String>,
    port: Option<u16>,
    domain: Option<String>,
    socket: Option<UdpSocket>,
    reg_timeout: Option<Instant>,
    client: Option<SoftPhone>,
    invitations: Vec<InviteHelper>
}

impl SipSessionProvider {

    pub fn new() -> SipSessionProvider {
        SipSessionProvider {
            acc: None,
            address: None,
            port: None,
            domain: None,
            socket: None,
            reg_timeout: None,
            client: None,
            invitations: vec![]
        }
    }

    pub async fn connect<'a>(&mut self, acc: Account, ctx: &mut ServerCtx<'a>) -> NirahResult<()> {
        let _ip_interface = default_ip_interface();

        let interface = __context_config_get_string!(ctx, _ip_interface)?;
        let address = ctx.address_manager
                        .network_from_name(&interface)
                        .unwrap_or("0.0.0.0".into());
        let port = ctx.address_manager.port();
        self.domain = Some(format!("{}:{}", address, port));
        self.address = Some(address);
        self.acc = Some(acc.clone());
        let socket = UdpSocket::bind(&self.domain.clone().unwrap()).await?;
        let mut client = SoftPhone::new(self.local_uri()?, self.account_uri()?);
        let _reg = client.registry_mut();
        _reg.username(&acc.username);
        _reg.password(&acc.password);
        self.port = Some(port);
        self.socket = Some(socket);
        self.client = Some(client);
        self.register().await?;
        Ok(())
    }

    fn local_uri(&self) -> NirahResult<Uri> {
        let domain = unwrap_or_else_not_connected!(self, domain, "Local Domain Not Set");
        let domain_str = format!("{} ", domain);
        let domain = parse_domain(domain_str.as_ref())?;

        let account = unwrap_or_else_not_connected!(self, acc, "Local Uri has not been set");

        Ok(Uri::sip(domain.1)
            .auth(UriAuth::new(&account.username))
            .parameter(Param::Transport(Transport::Udp))
        )
    }

    fn account_uri(&self) -> NirahResult<Uri> {
        let account = unwrap_or_else_not_connected!(self, acc, "Local Uri has not been set");
        let domain_str = format!("{} ", account.host);
        let domain = parse_domain(domain_str.as_ref())?;

        Ok(Uri::sip(domain.1)
            .auth(UriAuth::new(&account.username)))
    }
}

impl Provider for SipSessionProvider {
    fn nirah_provider_identifier(&self) -> &'static str {
        "SipSessionProvider"
    }

    fn nirah_provider_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn required_config_variables(&self) -> NirahResult<Vec<(VariableKey, Option<VariableValue>)>> {
        Ok(vec![
            (keys::default_ip_interface(), Some(keys::default_ip_interface_value()))
        ])
    }
}

pub(crate) fn not_connected<T>(msg: &'static str) -> NirahResult<T> {
    Err(io::Error::new(io::ErrorKind::NotConnected, msg).into())
}
