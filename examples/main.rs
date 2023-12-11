use stockfish::{game::Game, Context};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let mut cx = Context::new("./stockfish-ubuntu-x86-64").await?;
    let mut game = Game::new(&mut cx).await?;

    while game.result().is_none() {
        let calc = game.calculate().start().await?;
        // let _ = wait.next_line().await?;

        let (best_move, is_legal) = calc.stop_and_make_best_move().await?;
        println!("{best_move}");
        if !is_legal {
            panic!("Illegal move")
        }
    }

    println!("{:?}", game.result().unwrap());
    return Ok(());
}
