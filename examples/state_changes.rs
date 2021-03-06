use futures::stream::StreamExt; // for .next()
use std::error::Error;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use mpd_client::{commands, Client, Subsystem};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // The client is used to issue commands, and state_changes is an async stream of state change
    // notifications
    let (client, mut state_changes) = Client::connect_to("localhost:6600").await?;

    // You can also connect to Unix sockets
    // let (client, mut state_changes) = Client::connect_unix("/run/user/1000/mpd").await?;

    // Get the song playing right as we connect
    print_current_song(&client).await?;

    // Wait for state change notifications being emitted by MPD
    while let Some(subsys) = state_changes.next().await {
        let subsys = subsys?;

        if subsys == Subsystem::Player {
            print_current_song(&client).await?;
        }
    }

    Ok(())
}

async fn print_current_song(client: &Client) -> Result<(), Box<dyn Error>> {
    match client.command(commands::CurrentSong).await? {
        Some(song_in_queue) => {
            println!(
                "\"{}\" by \"{}\"",
                song_in_queue.song.title().unwrap_or(""),
                song_in_queue.song.artists().join(", "),
            );
        }
        None => println!("(none)"),
    }

    Ok(())
}
