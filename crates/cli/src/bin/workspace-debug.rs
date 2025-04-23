use anyhow::Result;
use clap::{Parser, Subcommand};
use sublime_workspace_cli::common::daemon::DaemonManager;
use sublime_workspace_cli::ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test daemon connectivity
    TestConnection {
        /// Number of times to test the connection
        #[arg(short, long, default_value = "5")]
        count: usize,

        /// Delay between tests in milliseconds
        #[arg(short, long, default_value = "500")]
        delay: u64,
    },

    /// Send a specific command to the daemon
    SendCommand {
        /// Command to send
        command: String,

        /// Arguments to pass with the command
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    // Initialize the UI system
    ui::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Setup logging
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // Create a daemon manager with default settings
    let daemon_manager = DaemonManager::default();

    match cli.command {
        Some(Commands::TestConnection { count, delay }) => {
            test_connection(&daemon_manager, count, delay)?;
        }
        Some(Commands::SendCommand { command, args }) => {
            send_command(&daemon_manager, &command, &args)?;
        }
        None => {
            println!("{}", ui::section_header("Workspace Debug Utility"));
            println!("Available commands:");
            println!("{}", ui::command_example("workspace-debug test-connection"));
            println!("{}", ui::command_example("workspace-debug send-command <cmd> [args...]"));
        }
    }

    Ok(())
}

fn test_connection(daemon_manager: &DaemonManager, count: usize, delay: u64) -> Result<()> {
    println!("{}", ui::section_header("Testing Daemon Connection"));

    let mut table_rows = Vec::new();
    let mut success_count = 0;

    for i in 1..=count {
        println!("{}", ui::info(&format!("Test {}/{}: connecting to daemon...", i, count)));

        let start = std::time::Instant::now();
        let result = daemon_manager.send_command("ping", &[]);
        let duration = start.elapsed().as_millis();

        match result {
            Ok(response) => {
                println!(
                    "{}",
                    ui::success(&format!("Received response in {}ms: {}", duration, response))
                );
                table_rows.push(vec![
                    i.to_string(),
                    "Success".to_string(),
                    format!("{}ms", duration),
                    response,
                ]);
                success_count += 1;
            }
            Err(e) => {
                println!(
                    "{}",
                    ui::error(&format!("Connection failed after {}ms: {}", duration, e))
                );
                table_rows.push(vec![
                    i.to_string(),
                    "Failed".to_string(),
                    format!("{}ms", duration),
                    e.to_string(),
                ]);
            }
        }

        if i < count {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
    }

    // Display results in a table
    println!(
        "\n{}",
        ui::highlight(&format!("Connection Test Results: {}/{} successful", success_count, count))
    );

    let headers = vec![
        "Test #".to_string(),
        "Status".to_string(),
        "Duration".to_string(),
        "Response/Error".to_string(),
    ];
    let tabular = ui::Tabular { headers, rows: table_rows };

    let options = ui::TabularOptions {
        title: Some("Connection Tests".to_string()),
        headers_in_columns: false,
        border_color: Some(tabled::settings::Color::FG_CYAN),
        header_color: Some(tabled::settings::Color::FG_YELLOW),
        header_title: None,
        footer_title: Some(format!("Success Rate: {}%", (success_count * 100) / count)),
    };

    println!("{}", ui::create_tabular(&tabular, &options));

    Ok(())
}

fn send_command(daemon_manager: &DaemonManager, command: &str, args: &[String]) -> Result<()> {
    println!("{}", ui::section_header(&format!("Sending Command: {}", command)));

    match daemon_manager.send_command(command, args) {
        Ok(response) => {
            println!("{}", ui::success("Command succeeded"));
            println!("{}", ui::key_value("Response", &response));
        }
        Err(e) => {
            println!("{}", ui::error(&format!("Command failed: {}", e)));
        }
    }

    Ok(())
}
