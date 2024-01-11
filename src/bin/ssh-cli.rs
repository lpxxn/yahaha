use std::env;
use std::sync::Arc;
use std::io::Write;
use thrussh::*;
use thrussh_keys::*;
use futures::Future;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(current_dir) = env::current_dir() {
        println!("The current directory is {}", current_dir.display());
    }
    let pem = std::fs::read_to_string("test.pem")?;
    // let mut ssh = Session::connect(&pem, "root", "127.0.0.1:7655").await?;
    let mut ssh = Session::connect2("li", "123", "127.0.0.1:7655").await?;
    let r = ssh.call("whoami").await?;
    assert!(r.success());
    assert_eq!(r.output(), "li\n");
    ssh.close().await?;
    Ok(())
}

struct Client {}

impl client::Handler for Client {
    type Error = thrussh::Error;
    type FutureUnit = futures::future::Ready<Result<(Self, client::Session), Self::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, bool), Self::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        futures::future::ready(Ok((self, b)))
    }
    fn finished(self, session: client::Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, session)))
    }
    fn check_server_key(self, _server_public_key: &key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }
}

pub struct Session {
    session: client::Handle<Client>,
}

impl Session {
    async fn connect(
        pem: &str,
        user: impl Into<String>,
        addr: impl std::net::ToSocketAddrs,
    ) -> Result<Self> {
        let key_pair = decode_secret_key(pem, None)?;
        let config = client::Config::default();
        let config = Arc::new(config);
        let sh = Client {};
        let mut agent = agent::client::AgentClient::connect_env().await?;
        agent.add_identity(&key_pair, &[]).await?;
        let mut identities = agent.request_identities().await?;
        let mut session = client::connect(config, addr, sh).await?;
        let pubkey = identities.pop().unwrap();
        let (_, auth_res) = session.authenticate_future(user, pubkey, agent).await;
        let _auth_res = auth_res?;
        Ok(Self { session })
    }

    async fn connect2(
        user: impl Into<String>,
        pwd: impl Into<String>,
        addr: impl std::net::ToSocketAddrs,
    ) -> Result<Self> {
        let config = client::Config::default();
        let config = Arc::new(config);
        let sh = Client {};
        let mut session = client::connect(config, addr, sh).await?;
        let ok = session.authenticate_password(user, pwd).await?;
        if !ok {
            return Err(anyhow::anyhow!("auth failed"));
        }
        Ok(Self { session })
    }

    async fn call(&mut self, command: &str) -> Result<CommandResult> {
        let is_closed = self.session.is_closed();
        println!("is_closed: {}", is_closed);
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut output = Vec::new();
        let mut code = None;
        while let Some(msg) = channel.wait().await {
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    output.write_all(&data).unwrap();
                }
                thrussh::ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }
                _ => {}
            }
        }
        Ok(CommandResult { output, code })
    }

    async fn close(&mut self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

struct CommandResult {
    output: Vec<u8>,
    code: Option<u32>,
}

impl CommandResult {
    fn output(&self) -> String {
        String::from_utf8_lossy(&self.output).into()
    }

    fn success(&self) -> bool {
        self.code == Some(0)
    }
}
