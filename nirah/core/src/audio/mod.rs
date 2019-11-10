use async_trait::async_trait;

use crate::prelude::*;

mod rodio;
pub use self::rodio::RodioAudioProvider;

#[async_trait]
pub trait AudioProvider: Provider {

    async fn start_ringing<'a>(&mut self, cfg: &mut ConfigFuture) -> NirahResult<()>;

    async fn stop_ringing<'a>(&mut self, cfg: &mut ConfigFuture) -> NirahResult<()>;
}
