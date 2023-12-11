use futures::Stream;
use pin_project::pin_project;
use std::task::{ready, Poll};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::ChildStdout,
};
use vampirc_uci::{parse_one, UciMessage};

#[derive(Debug)]
#[pin_project]
pub struct Messages {
    #[pin]
    lines: Lines<BufReader<ChildStdout>>,
}

impl Messages {
    pub fn new(stdout: ChildStdout) -> Self {
        return Self {
            lines: BufReader::new(stdout).lines(),
        };
    }
}

impl Stream for Messages {
    type Item = std::io::Result<UciMessage>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let Some(line) = ready!(this.lines.poll_next_line(cx))? else {
            return Poll::Ready(None);
        };
        return Poll::Ready(Some(Ok(parse_one(&line))));
    }
}
