use tokio::net::UdpSocket;
use tokio::time::Instant;
use libsip::client::SoftPhone;
use libsip::client::InviteHelper;
use libsip::client::HeaderWriteConfig;
use libsip::uri::Uri;
use libsip::uri::UriAuth;
use libsip::uri::Param;
use libsip::uri::parse_domain;
use libsip::core::Transport;
use libsdp::SdpMediaType;
use libsdp::SdpTransport;
use libsdp::SdpCodecIdentifier;
use libsdp::SdpSanitizer;
use libsdp::SdpSanitizerConfig;

use crate::prelude::*;
use crate::config::keys::default_ip_interface;


#[macro_use]
mod macros;
pub mod errors;
mod message;
mod register;
mod session;
mod invite;
mod bye;
mod cancel;

pub struct SipSessionProvider {
    acc: Option<Account>,
    address: Option<String>,
    port: Option<u16>,
    domain: Option<String>,
    socket: Option<UdpSocket>,
    reg_timeout: Option<Instant>,
    client: Option<SoftPhone>,
    invitations: Vec<InviteHelper>,
    active: Vec<InviteHelper>,
    sanitizer: SdpSanitizer,
    header_config: HeaderWriteConfig
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
            invitations: vec![],
            active: vec![],
            sanitizer: SdpSanitizer::new(SdpSanitizerConfig {
                allowed_codecs: vec![SdpCodecIdentifier(0)],
                allowed_transports: vec![SdpTransport::RtpAvp],
                allowed_media_types: vec![SdpMediaType::Audio]
            }),
            header_config: HeaderWriteConfig::default()
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
        let socket = UdpSocket::bind(self.domain.clone().unwrap()).await?;
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

    fn required_config_variables(&self) -> NirahResult<Vec<ConfigDefinition>> {
        Ok(vec![
            ConfigDefinition {
                key: keys::default_ip_interface(),
                default: Some(keys::default_ip_interface_value()),
                description: Some("Ip interface used for SIP communications".into())
            }
        ])
    }
}
