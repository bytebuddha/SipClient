use nirah_sdp::SdpOffer;
use nirah_sdp::SdpNetworkType;
use nirah_sdp::SdpAddressType;
use nirah_sdp::SdpConnection;
use nirah_sdp::SdpMedia;
use nirah_sdp::SdpMediaFormat;
use nirah_sdp::SdpMediaType;
use nirah_sdp::SdpProtocol;
use nirah_sdp::SdpSessionAttributes;
use nirah_sdp::Codec;
use nirah_sdp::parse_sdp_offer;

use crate::prelude::*;
use crate::config::keys::{ default_ip_interface, default_ip_interface_value };

impl SipSessionProvider {
    pub(crate) async fn handle_invite<'a>(&mut self, msg: nirah_sip::SipMessage, ctx: SessionCtx<'a>) -> NirahResult<()> {
        if let nirah_sip::SipMessage::Request { uri, headers, body, .. } = msg {
            let helper = nirah_sip::client::InviteHelper::new(uri.clone(), headers, body)?;
            let socket = unwrap_mut_or_else_not_connected!(self, socket, "Socket not connected");
            let account = unwrap_or_else_not_connected!(self, acc, "Account not connected");
            let ring_req = helper.ringing()?;
            let data = format!("{}", ring_req);
            socket.send_to(data.as_ref(), &account.get_socket_address()).await?;
            ctx.audio.start_ringing(ctx.config).await?;
            if let Some(contact) = ctx.contacts.get_contact_from_uri(uri).await? {
                let new_msg = NewTransactionEvent {
                    account: account.id,
                    contact: contact.id,
                    sent: false,
                    time: chrono::Utc::now().naive_utc(),
                    data: NewTransactionEventData::NewInvitation {

                    }
                };
                let log_id = ctx.database.log(contact.id, new_msg).await?;
                let logged_message = ctx.database.get_log(contact.id, log_id).await?.unwrap();
                let invite_id = self.invitations.len();
                self.invitations.push(helper);
                let params = NotifierParams {
                    invite_id: Some(invite_id),
                };
                let arg = NotifierArgument { account: &account, contact: &contact, params, msg: &logged_message };
                ctx.notifier.new_transaction(arg).await?;
            } else {
                let from_header = helper.from()?;
                debug!("Creating new contact for: {:?}, ", from_header.uri);
                let id = ctx.contacts.create_contact(NewContact { display_name: from_header.display_name, uri: from_header.uri }).await?;
                let logged_message = NewTransactionEvent {
                    account: account.id,
                    contact: id,
                    sent: false,
                    time: chrono::Utc::now().naive_utc(),
                    data: NewTransactionEventData::NewInvitation {

                    }
                };
                let log_id = ctx.database.log(id, logged_message).await?;
                let msg = ctx.database.get_log(id, log_id).await?.unwrap();
                let contact = ctx.contacts.get_contact(id).await?.unwrap();
                let invite_id = self.invitations.len();
                self.invitations.push(helper);
                let params = NotifierParams {
                    invite_id: Some(invite_id),
                };
                let arg = NotifierArgument { account: &account, contact: &contact, params, msg: &msg };
                ctx.notifier.new_transaction(arg).await?;
            }
        } else {
            warn!("handle_invite passed a sip message that was not a request.")
        }
        Ok(())
    }

    pub async fn accept_invite<'a>(&mut self, mut ctx: SessionCtx<'a>, invite: usize) -> NirahResult<()> {
        if let Some(invitation) = self.invitations.get(invite) {
            let (_, possible_sdp) = parse_sdp_offer(&invitation.body)?;
            if let Some(response_sdp) = self.get_response_sdp(&mut ctx, &possible_sdp).await? {
                 let event = StreamingEvent {
                     inputs: vec![possible_sdp],
                     outputs: vec![response_sdp]
                 };
                 ctx.streaming.handle_session(streaming_ctx!(ctx), event).await?;
            } else {
                warn!("Failed to create response SDP Message: {:?}", possible_sdp);
            }
        } else {
            warn!("Attempted to accept a non existant invite");
        }
        Ok(())
    }

    async fn get_response_sdp<'a>(&self, ctx: &mut SessionCtx<'a>, sdp: &SdpOffer) -> NirahResult<Option<SdpOffer>> {
        let mut is_valid = false;
        for media in &sdp.media {
            for format in &media.formats {
                if format.codec == Codec::Pcmu {
                    is_valid = true;
                }
            }
        }
        if is_valid {
            let _ip_interface = default_ip_interface();
            let _default_ip_interface = default_ip_interface_value();
            let interface = context_config_get_string!(ctx, _ip_interface, _default_ip_interface)?;
            let address = ctx.address_manager
                            .network_from_name(&interface)
                            .expect("Failed to get current ip address");
            let connection = SdpConnection {
                network_type: SdpNetworkType::Internet,
                address_type: SdpAddressType::Ipv4,
                address: address
            };
            let new = SdpOffer::new(sdp.origin.clone(), sdp.name.clone())
                            .add_optional_attribute(SdpSessionAttributes::Connection(connection))
                            .add_media(
                                SdpMedia::new(SdpMediaType::Audio, ctx.address_manager.port() as u32, SdpProtocol::RtpAvp)
                                    .add_format(SdpMediaFormat::new(Codec::Pcmu))
                            );
            Ok(Some(new))
        } else {
            Ok(None)
        }
    }
}
