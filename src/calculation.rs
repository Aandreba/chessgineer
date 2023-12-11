use crate::game::Game;
use futures::{Future, Stream};
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{ready, Poll},
};
use tokio::io::AsyncWriteExt;
use vampirc_uci::{Serializable, UciMessage, UciMove, UciTimeControl};

#[pin_project]
pub struct Calculation<'a> {
    game: &'a mut Game<'a>,
    #[pin]
    timeout: TimeoutState<'a>,
    sink: Vec<UciMessage>,
}

impl<'a> Calculation<'a> {
    pub async fn stop(self) -> std::io::Result<UciMove> {
        self.game.cx.stdin.write_all(b"stop\n").await?;
        return self.await;
    }
}

impl<'a> Stream for Calculation<'a> {
    type Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

/// Returns the calculation's best move
impl<'a> Future for Calculation<'a> {
    type Output = std::io::Result<UciMove>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let _ = this.timeout.poll(this.game, cx)?;

        match ready!(this.game.cx.messages.poll_next_unpin(cx)).transpose()? {
            Some(UciMessage::BestMove { best_move, ponder }) => todo!(),
            Some(UciMessage::Info(info)) => {}
            None => todo!(),
        }

        todo!()
    }
}

#[derive(Debug)]
pub struct CalculationBuilder<'a> {
    pub(crate) game: &'a mut Game<'a>,
    pub(crate) white_time: Option<MoveTime>,
    pub(crate) black_time: Option<MoveTime>,
    pub(crate) timeout: Option<vampirc_uci::Duration>,
}

impl<'a> CalculationBuilder<'a> {
    pub async fn start(self) -> std::io::Result<Calculation<'a>> {
        let (time_control, timeout) = match (self.white_time, self.black_time, self.timeout) {
            (None, None, None) => (UciTimeControl::Infinite, None),
            (None, None, Some(timeout)) => (UciTimeControl::MoveTime(timeout), None),
            (white, black, timeout) => (
                UciTimeControl::TimeLeft {
                    white_time: white.map(|x| x.remaining),
                    black_time: black.map(|x| x.remaining),
                    white_increment: white.and_then(|x| x.increment),
                    black_increment: black.and_then(|x| x.increment),
                    moves_to_go: None,
                },
                timeout,
            ),
        };

        let mut go_cmd = UciMessage::Go {
            time_control: Some(time_control),
            search_control: None,
        }
        .serialize();
        go_cmd.push('\n');
        self.game.cx.stdin.write_all(go_cmd.as_bytes()).await?;

        return Ok(Calculation {
            game: self.game,
            timeout: match timeout {
                Some(dur) => TimeoutState::Waiting(tokio::time::sleep(dur.to_std().unwrap())),
                None => TimeoutState::Done,
            },
            sink: Vec::new(),
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct MoveTime {
    remaining: vampirc_uci::Duration,
    increment: Option<vampirc_uci::Duration>,
}

#[pin_project(project = TimeoutStateProj)]
enum TimeoutState<'a> {
    Waiting(#[pin] tokio::time::Sleep),
    Stopping(Pin<Box<dyn 'a + Future<Output = std::io::Result<()>> + Send + Sync>>),
    Done,
}

impl<'a> TimeoutState<'a> {
    fn poll(
        mut self: Pin<&mut Self>,
        game: &'a mut Game<'a>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            match self.project() {
                TimeoutStateProj::Waiting(sleep) => {
                    ready!(sleep.poll(cx));
                    self.set(TimeoutState::Stopping(Box::pin(
                        game.cx.stdin.write_all(b"stop\n"),
                    )))
                }
                TimeoutStateProj::Stopping(stop) => {
                    ready!(stop.as_mut().poll(cx))?;
                    self.set(TimeoutState::Done);
                    return Poll::Ready(Ok(()));
                }
                TimeoutStateProj::Done => return Poll::Ready(Ok(())),
            }
        }
    }
}
