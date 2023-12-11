use crate::game::Game;
use chess::ChessMove;
use futures::{Future, Stream, StreamExt};
use pin_project::pin_project;
use std::{
    collections::VecDeque,
    io::ErrorKind,
    pin::Pin,
    task::{ready, Poll},
    time::Duration,
};
use tokio::io::AsyncWriteExt;
use vampirc_uci::{Serializable, UciFen, UciInfoAttribute, UciMessage, UciTimeControl};

#[pin_project]
pub struct Calculation<'a, 'b> {
    game: &'b mut Game<'a>,
    sink: VecDeque<UciInfoAttribute>,
    best_move: Option<ChessMove>,
}

impl<'a, 'b> Calculation<'a, 'b> {
    pub async fn stop_and_make_best_move(mut self) -> std::io::Result<(ChessMove, bool)> {
        self.game.cx.stdin.write_all(b"stop\n").await?;
        let best_move = (&mut self).await?;
        let is_legal = self.game.make_move(best_move);
        return Ok((best_move, is_legal));
    }

    pub async fn make_best_move(mut self) -> std::io::Result<(ChessMove, bool)> {
        let best_move = (&mut self).await?;
        let is_legal = self.game.make_move(best_move);
        return Ok((best_move, is_legal));
    }

    pub async fn stop(self) -> std::io::Result<ChessMove> {
        self.game.cx.stdin.write_all(b"stop\n").await?;
        return self.await;
    }
}

/// Returns the information provided by the engine
impl<'a, 'b> Stream for Calculation<'a, 'b> {
    type Item = std::io::Result<UciInfoAttribute>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if let Some(next) = this.sink.pop_front() {
            return Poll::Ready(Some(Ok(next)));
        }

        loop {
            match ready!(this.game.cx.messages.poll_next_unpin(cx)).transpose()? {
                Some(UciMessage::BestMove { best_move, .. }) => {
                    *this.best_move = Some(best_move);
                    return Poll::Ready(None);
                }
                Some(UciMessage::Info(info)) => {
                    let mut info = VecDeque::from(info);
                    let Some(next) = info.pop_front() else {
                        continue;
                    };

                    this.sink.append(&mut VecDeque::from(info));
                    return Poll::Ready(Some(Ok(next)));
                }
                _ => todo!(),
            };
        }
    }
}

/// Returns the calculation's best move
impl<'a, 'b> Future for Calculation<'a, 'b> {
    type Output = std::io::Result<ChessMove>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        if let Some(result) = this.best_move.take() {
            return Poll::Ready(Ok(result));
        }

        loop {
            match ready!(this.game.cx.messages.poll_next_unpin(cx)).transpose()? {
                Some(UciMessage::BestMove { best_move, .. }) => return Poll::Ready(Ok(best_move)),
                Some(UciMessage::Info(info)) => this.sink.append(&mut VecDeque::from(info)),
                _ => todo!(),
            };
        }
    }
}

#[derive(Debug)]
pub struct CalculationBuilder<'a, 'b> {
    pub(crate) game: &'b mut Game<'a>,
    pub(crate) white_time: Option<MoveTime>,
    pub(crate) black_time: Option<MoveTime>,
    pub(crate) timeout: Option<vampirc_uci::Duration>,
}

impl<'a, 'b> CalculationBuilder<'a, 'b> {
    pub fn white_time(mut self, dur: Duration, increment: Option<Duration>) -> Self {
        self.white_time = Some(MoveTime {
            remaining: std_to_vampiri_saturating(dur),
            increment: increment.map(std_to_vampiri_saturating),
        });
        self
    }

    pub fn black_time(mut self, dur: Duration, increment: Option<Duration>) -> Self {
        self.black_time = Some(MoveTime {
            remaining: std_to_vampiri_saturating(dur),
            increment: increment.map(std_to_vampiri_saturating),
        });
        self
    }

    pub fn timeout(mut self, dur: Duration) -> Self {
        self.timeout = Some(std_to_vampiri_saturating(dur));
        self
    }

    pub async fn best_move(self) -> std::io::Result<ChessMove> {
        return match (self.white_time, self.black_time, self.timeout) {
            (None, None, None) => Err(std::io::Error::new(ErrorKind::Other, "no timeout set")),
            _ => self.start().await?.await,
        };
    }

    pub async fn start(self) -> std::io::Result<Calculation<'a, 'b>> {
        let time_control = match (self.white_time, self.black_time, self.timeout) {
            (None, None, None) => UciTimeControl::Infinite,
            (None, None, Some(timeout)) => UciTimeControl::MoveTime(timeout),
            (white, black, timeout) => UciTimeControl::TimeLeft {
                white_time: white
                    .zip(timeout)
                    .map(|(x, y)| vampirc_uci::Duration::min(x.remaining, y)),
                black_time: black
                    .zip(timeout)
                    .map(|(x, y)| vampirc_uci::Duration::min(x.remaining, y)),
                white_increment: white.and_then(|x| x.increment),
                black_increment: black.and_then(|x| x.increment),
                moves_to_go: None,
            },
        };

        // Update position
        let mut position_cmd = UciMessage::Position {
            startpos: false,
            fen: Some(UciFen(self.game.current_position().to_string())),
            moves: Vec::new(),
        }
        .serialize();
        position_cmd.push('\n');
        self.game
            .cx
            .stdin
            .write_all(position_cmd.as_bytes())
            .await?;

        // Start calculating
        let mut go_cmd = UciMessage::Go {
            time_control: Some(time_control),
            search_control: None,
        }
        .serialize();
        go_cmd.push('\n');
        self.game.cx.stdin.write_all(go_cmd.as_bytes()).await?;

        return Ok(Calculation {
            game: self.game,
            sink: VecDeque::new(),
            best_move: None,
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MoveTime {
    remaining: vampirc_uci::Duration,
    increment: Option<vampirc_uci::Duration>,
}

fn std_to_vampiri_saturating(dur: std::time::Duration) -> vampirc_uci::Duration {
    vampirc_uci::Duration::from_std(dur).unwrap_or(vampirc_uci::Duration::max_value())
}
