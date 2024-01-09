use thrussh::*;
use thrussh::client::Session;
use futures::Future;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let config = thrussh::client::Config

}

struct Client {}

impl client::Handler for Client {
    type Error = anyhow::Error;
    type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        todo!()
    }

    fn finished(self, session: Session) -> Self::FutureUnit {
        todo!()
    }
}