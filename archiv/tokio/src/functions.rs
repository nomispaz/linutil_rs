use std::{env::args, process::Stdio, sync::Arc};

use anyhow::Ok;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

// Function to spawn the bash command and send each line of output over the channel
pub async fn run_command(
    tx: Sender<String>,
    rx: Arc<Mutex<Receiver<String>>>,
    commands: Vec<String>,
) -> anyhow::Result<()> {
    let joined_command = commands.join("; ");
    // Run the Bash command
    let mut cmd = Command::new("bash")
        .arg("-c")
        .arg(joined_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Clone the receiver for input piping
    if let Some(mut stdin) = cmd.stdin.take() {
        let rx_clone = Arc::clone(&rx);

        tokio::spawn(async move {
            let mut guard = rx_clone.lock().await;
            loop {
                let line = guard.recv().await;
                match line {
                    Some(msg) => {
                        if stdin.write_all(msg.as_bytes()).await.is_err() {
                            eprintln!("Error writing to channel");
                            break;
                        }
                        if stdin.flush().await.is_err() {
                            eprintln!("Error while flushing channel");
                            break;
                        }
                    }
                    _ => {
                        eprintln!("Channel has been closed");
                        break;
                    }
                }
            }
        });
    }

    // Check if we can capture the stdout
    if let Some(stdout) = cmd.stdout.take() {
        let reader = BufReader::new(stdout);

        // Send each line as it is available to the main thread
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            tx.send(line).await?;
        }
    }

    // Ensure the command completes
    cmd.wait().await?;
    tx.send("cmd finished".to_string()).await?;
    Ok(())
}
