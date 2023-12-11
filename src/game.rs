use crate::{calculation::CalculationBuilder, Context};
use chess::Board;
use pin_project::pin_project;
use std::ops::{Deref, DerefMut};
use tokio::io::AsyncWriteExt;

/* GAME */
#[derive(Debug)]
#[pin_project]
pub struct Game<'a> {
    pub(crate) cx: &'a mut Context,
    game: chess::Game,
}

impl<'a> Game<'a> {
    pub async fn new(cx: &'a mut Context) -> std::io::Result<Self> {
        return Self::with_board(Board::default(), cx).await;
    }

    pub async fn with_board(board: Board, cx: &'a mut Context) -> std::io::Result<Self> {
        return Self::with_game(chess::Game::new_with_board(board), cx).await;
    }

    pub async fn with_game(game: chess::Game, cx: &'a mut Context) -> std::io::Result<Self> {
        cx.stdin.write_all(b"ucinewgame\n").await?;
        cx.wait_readyness().await?;
        return Ok(Self { cx, game });
    }

    #[inline]
    pub fn state(&self) -> &chess::Game {
        self
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut chess::Game {
        self
    }

    pub fn calculate(&mut self) -> CalculationBuilder<'a, '_> {
        return CalculationBuilder {
            game: self,
            white_time: None,
            black_time: None,
            timeout: None,
        };
    }
}

impl<'a> Deref for Game<'a> {
    type Target = chess::Game;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl<'a> DerefMut for Game<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game
    }
}
