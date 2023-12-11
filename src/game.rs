use crate::{calculation::CalculationBuilder, Context};
use pin_project::pin_project;
use std::time::Duration;

/* GAME */
#[derive(Debug)]
#[pin_project]
pub struct Game<'a> {
    pub(crate) cx: &'a mut Context,
}

impl<'a> Game<'a> {
    pub fn calculate(&mut self) -> CalculationBuilder<'_> {
        return CalculationBuilder {
            game: self,
            white_time: None,
            black_time: None,
            timeout: None,
        };
    }
}

pub struct GameBuilder<'a> {
    cx: &'a mut Context,
    white_time: Option<Duration>,
    black_time: Option<Duration>,
    timeout: Option<Duration>,
}

impl<'a> GameBuilder<'a> {
    pub fn new(cx: &'a mut Context) -> Self {
        return Self {
            cx,
            white_time: None,
            black_time: None,
            timeout: None,
        };
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Game<'a> {
        todo!()
    }
}
