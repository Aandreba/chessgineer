use futures::StreamExt;
use game::Game;
use messages::Messages;
use std::{ffi::OsStr, io::ErrorKind, process::Stdio};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, ChildStdin, Command},
};
use vampirc_uci::{UciMessage, UciOptionConfig};

pub mod calculation;
pub mod game;
pub mod messages;

#[derive(Debug)]
pub struct Context {
    stdin: ChildStdin,
    messages: Messages,
    options: Vec<UciOptionConfig>,
}

impl Context {
    pub async fn new(path: impl AsRef<OsStr>) -> std::io::Result<Self> {
        Builder::new().build(path).await
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn options(&self) -> &[UciOptionConfig] {
        &self.options
    }

    pub async fn start_game(&mut self) -> std::io::Result<Game<'_>> {
        self.stdin.write_all(b"ucinewgame\n").await?;
        self.wait_readyness().await?;
        return Ok(Game { cx: self });
    }

    pub async fn wait_readyness(&mut self) -> std::io::Result<()> {
        self.stdin.write_all(b"isready\n").await?;
        return match self.messages.next().await.transpose()? {
            Some(UciMessage::ReadyOk) => return Ok(()),
            Some(other) => Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Unexpected response: {other}"),
            )),
            None => Err(ErrorKind::UnexpectedEof.into()),
        };
    }
}

pub struct Builder {
    debug_mode: bool,
}

impl Builder {
    pub fn new() -> Self {
        return Self {
            debug_mode: cfg!(any(tests, debug_assertions)),
        };
    }

    pub fn debug_mode(&mut self, debug_mode: bool) -> &mut Self {
        self.debug_mode = debug_mode;
        self
    }

    pub async fn build(&self, path: impl AsRef<OsStr>) -> std::io::Result<Context> {
        let Child { stdin, stdout, .. } = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut this = Context {
            stdin: stdin.unwrap(),
            messages: Messages::new(stdout.unwrap()),
            options: Vec::new(),
        };

        this.stdin.write_all(b"uci\n").await?;
        while let Some(resp) = this.messages.next().await.transpose()? {
            match resp {
                UciMessage::Option(opt) => this.options.push(opt),
                UciMessage::UciOk => break,
                // skip
                _ => continue,
            }
        }

        this.wait_readyness().await?;
        return Ok(this);
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
