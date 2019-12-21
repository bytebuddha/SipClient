use async_trait::async_trait;
use libsdp::SdpEncoding;
use libsdp::SdpCodecIdentifier;
use gstreamer::Caps;
use gstreamer::Pipeline;
use gstreamer::Element;
use gstreamer::ElementExt;
use gstreamer::ElementExtManual;
use gstreamer::GstBinExtManual;
use gstreamer::ElementFactory;
use gstreamer::ToValue;
use gstreamer::prelude::ObjectExt;

use nirah_core::prelude::*;

pub struct GSTreamerProvider {

}

impl GSTreamerProvider {

    pub fn new() -> NirahResult<GSTreamerProvider> {
        gstreamer::init().unwrap();
        Ok(GSTreamerProvider {})
    }

    pub fn audio_input_element(&self) -> NirahResult<Element> {
        Ok(ElementFactory::make("pulsesrc", None).expect("Failed to create audio input element"))
    }

    pub fn audio_output_element(&self) -> NirahResult<Element> {
        Ok(ElementFactory::make("pulsesink", None).expect("Failed to create audio output element"))
    }

    pub fn udp_sink(&self, address: String, port: u32) -> NirahResult<Element> {
        let elem = ElementFactory::make("udpsink", None).expect("Failed to open udpsink");
        elem.set_property("port", &(port as i32).to_value()).expect("Failed to udpsink port property");
        elem.set_property("host", &address.to_value()).expect("Failed to set udpsink host property");
        Ok(elem)
    }

    pub fn udp_src(&self, identifier: SdpCodecIdentifier, codec: SdpEncoding, clock_rate: u64, port: u32) -> NirahResult<Element> {
        let elem = ElementFactory::make("udpsrc", None).expect("Failed to open udpsrc");
        let elem_caps = Caps::new_simple("application/x-rtp", &[
            ("media", &"audio"),
            ("payload", &(identifier.0 as i32)),
            ("encoding-name", &format!("{}", codec)),
            ("clock-rate", &(clock_rate as i32))
        ]);
        elem.set_property("port", &(port as i32).to_value()).expect("Failed to set udpsrc port property");
        elem.set_property("caps", &elem_caps.to_value()).expect("Failed to set udpsrc caps property");
        Ok(elem)
    }

    pub fn get_rtp_encoder_element(&self, encoding: SdpEncoding) -> NirahResult<Element> {
        match encoding {
            SdpEncoding::Pcmu => Ok(ElementFactory::make("rtppcmupay", None).expect("Failed to create rtppcmupay element")),
            _ => { unimplemented!() }
        }
    }

    pub fn get_rtp_decoder_element(&self, encoding: SdpEncoding) -> NirahResult<Element> {
        match encoding {
            SdpEncoding::Pcmu => Ok(ElementFactory::make("rtppcmudepay", None).expect("Failed to create rtppcmudepay element")),
            _ => { unimplemented!() }
        }
    }

    pub fn get_audio_encoder_element(&self, encoding: SdpEncoding) -> NirahResult<Element> {
        match encoding {
            SdpEncoding::Pcmu => Ok(ElementFactory::make("mulawenc", None).expect("Failed to create mulawenc element")),
            _ => { unimplemented!() }
        }
    }

    pub fn get_audio_decoder_element(&self, encoding: SdpEncoding) -> NirahResult<Element> {
        match encoding {
            SdpEncoding::Pcmu => Ok(ElementFactory::make("mulawdec", None).expect("Failed to create mulawdec element")),
            _ => { unimplemented!() }
        }
    }
}

impl Provider for GSTreamerProvider {
    fn nirah_provider_identifier(&self) -> &'static str {
        "GSTreamerStreamingProvider"
    }

    fn nirah_provider_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn required_config_variables(&self) -> NirahResult<Vec<ConfigDefinition>> {
        Ok(vec![])
    }
}

#[async_trait]
impl StreamingProvider for GSTreamerProvider {
    async fn list_streams(&self) -> NirahResult<Vec<String>> {
        Ok(vec![])
    }

    async fn handle_streams<'a>(&mut self, _ctx: StreamingCtx<'a>, events: Vec<StreamingEvent>) -> NirahResult<()> {
        for event in events {
            debug!("Creating Stream: {:?}", event);
            match event {
                StreamingEvent::AudioSession {
                    call_id, local_port, remote_port,
                    remote_addr, codec, clock_rate,
                    identifier
                } => {
                    let udp_in = self.udp_src(identifier, codec.clone(), clock_rate, local_port)?;
                    let rtp_decoder = self.get_rtp_decoder_element(codec.clone())?;
                    let audio_decoder = self.get_audio_decoder_element(codec.clone())?;
                    let audio_output = self.audio_output_element()?;
                    let queue = ElementFactory::make("queue", None).expect("Failed to create queue element");

                    let audio_input = self.audio_input_element()?;
                    let audio_encoder = self.get_audio_encoder_element(codec.clone())?;
                    let rtp_encoder = self.get_rtp_encoder_element(codec)?;
                    let udp_out = self.udp_sink(remote_addr, remote_port)?;
                    let resample = ElementFactory::make("audioresample", None).expect("Failed to create audioresample element");
                    let queue2 = ElementFactory::make("queue", None).expect("Failed to create queue element");

                    let pipeline = Pipeline::new(None);
                    pipeline.add_many(
                        &[&audio_input, &queue2, &resample, &audio_encoder, &rtp_encoder, &udp_out]
                    ).expect("Failed to link elements to the pipeline");
                    pipeline.add_many(
                        &[&udp_in, &queue, &rtp_decoder, &audio_decoder, &audio_output]
                    ).expect("Failed to add elements to the pipeline");

                    Element::link_many(&[&audio_input, &queue2, &resample, &audio_encoder, &rtp_encoder, &udp_out]).expect("Failed to link elements");
                    Element::link_many(&[&udp_in, &queue, &rtp_decoder, &audio_decoder, &audio_output]).expect("Failed to link elements");

                    let bus = pipeline
                        .get_bus()
                        .expect("Pipeline1 without bus. Shouldn't happen!");
                    pipeline
                        .set_state(gstreamer::State::Playing)
                        .expect("Unable to set the pipeline to the `Playing` state");

                    for msg in bus.iter_timed(gstreamer::CLOCK_TIME_NONE) {
                    use gstreamer::MessageView;
                    match msg.view() {
                        MessageView::Eos(..) => break,
                        MessageView::Error(err) => {

                            println!("{:?}", err);
                        },
                        _ => (),
                    }
                }
                }
            }
        }
        Ok(())
    }

    async fn end_stream<'a>(&mut self, _ctx: StreamingCtx<'a>, _call_id: String) -> NirahResult<()> {
        Ok(())
    }
}
