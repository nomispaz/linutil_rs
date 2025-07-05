use std::{
    env::args,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

// Function to spawn the bash command and send each line of output over the channel
pub fn run_command(
    tx: Sender<String>,
    //rx: Receiver<String>,
    rx: Arc<Mutex<Receiver<String>>>,
    commands: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let joined_command = commands.join("; ");
    // Run the Bash command
    let mut cmd = Command::new("bash")
        .arg("-c")
        .arg(joined_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let rx_clone = Arc::clone(&rx);

    // run a receiver thread to listen to the stdin of the process
    let mut stdin = cmd.stdin.take().expect("Could't access stdin");
    // Move stdin into input thread (no cloning)
    let _stdin_input_thread = thread::spawn(move || {
        let rx = rx_clone.lock().unwrap();
        while let Ok(line) = rx.recv() {
            if stdin.write_all(line.as_bytes()).is_err() {
                break;
            }
            if stdin.flush().is_err() {
                break;
            }
        }
    });

    // Check if we can capture the stdout
    if let Some(stdout) = cmd.stdout.take() {
        let reader = BufReader::new(stdout);

        // Send each line as it is available to the main thread
        for line in reader.lines() {
            let line = line.unwrap();
            tx.send(line)?;
        }
    }

    // Ensure the command completes
    cmd.wait()?;
    Ok(())
}

pub fn run_command_wo_bash(
    tx: Sender<String>,
    commands: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let joined_command = commands.join("; ");
    // Run the Bash command
    let mut cmd = Command::new("ls")
        .arg("-l")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Check if we can capture the stdout
    if let Some(stdout) = cmd.stdout.take() {
        let reader = BufReader::new(stdout);

        // Send each line as it is available to the main thread
        for line in reader.lines() {
            let line = line.unwrap();
            tx.send(line)?;
        }
    }

    // Ensure the command completes
    cmd.wait()?;
    Ok(())
}
