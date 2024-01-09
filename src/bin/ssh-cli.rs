use std::sync::Arc;

use thrussh::*;
use thrussh::client::Session;
use thrussh_keys::*;
use futures::Future;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let config = thrussh::client::Config::default();
    let config = Arc::new(config);
    let sh = Client {};
}

struct Client {}

impl client::Handler for Client {
    type Error = anyhow::Error;
    type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        futures::future::ready(Ok((self, b)))
    }

    fn finished(self, session: client::Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, session)))
    }

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        println!("check_server_key: {:?}", server_public_key);
        self.finished_bool(true)
    }

    fn channel_open_confirmation(self, channel: ChannelId, max_packet_size: u32, window_size: u32, session: client::Session) -> Self::FutureUnit {
        println!("channel_open_confirmation: {:?}", channel);
        self.finished(session)
    }
    fn data(self, channel: ChannelId, data: &[u8], session: client::Session) -> Self::FutureUnit {
        println!("data on channel {:?}: {:?}", channel, std::str::from_utf8(data));
        self.finished(session)
    }
}