use std::{env::Args, fs::read_to_string, process::Stdio, sync::Arc};

use anyhow::Ok;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};

use crate::app::Config;

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
    // Use joinhandle so that we can close the receiver when the command-process ends.
    let stdin_writer_handle: Option<JoinHandle<()>> = if let Some(mut stdin) = cmd.stdin.take() {
        let rx_clone = Arc::clone(&rx);
        Some(tokio::spawn(async move {
            // lock the mutex so that this thread can write to it.
            let mut guard = rx_clone.lock().await;
            loop {
                // get messages from the receiver
                let line = guard.recv().await;
                match line {
                    Some(msg) => {
                        // write the message to stdin and therefore to the cmd process
                        let stdin_write_res = stdin.write_all(msg.as_bytes()).await;
                        if stdin_write_res.is_err() {
                            eprintln!("Error writing to channel: {:?}", stdin_write_res);
                            break;
                        }
                        // flush the output stream
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
        }))
    } else {
        None
    };

    // Check if we can capture the stdout
    if let Some(stdout) = cmd.stdout.take() {
        let reader = BufReader::new(stdout);

        // Send each line as it is available to the main thread
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            // send every output to the main process
            tx.send(line).await?;
        }
    }

    // Ensure the command completes
    cmd.wait().await?;
    // after the command completed, close the thread that handles the transfer of messages to stdin
    // of the now closed process
    if let Some(handle) = stdin_writer_handle {
        handle.abort();
        let _ = handle.await;
    }
    Ok(())
}

pub fn read_config(file_path: &str, args: &Args) -> anyhow::Result<Config> {
    let file_content = read_to_string(file_path)?;
    let mut config: Config = toml::from_str(&file_content)?;
    Ok(config)
}
