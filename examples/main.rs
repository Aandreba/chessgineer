use stockfish::Context;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let mut cx = Context::new("./stockfish-ubuntu-x86-64").await?;
    let mut game = cx.start_game().await?;
    println!("Hi");
    return Ok(());
}
