use console::Term;
use std::{env, time::Duration};
use sublime_workspace_cli::ui;

fn main() {
    // Initialize the UI system
    ui::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "boxes" => test_boxes(),
        "lists" => test_lists(),
        "messages" => test_messages(),
        "tables" => test_tables(),
        "progress" => test_progress(),
        "symbols" => test_symbols(),
        "styles" => test_styles(),
        "inputs" => test_inputs(),
        "help" => test_help(),
        "all" => test_all(),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Usage: cargo run --bin test-ui [component]");
    println!("Available components:");
    println!("  boxes     - Test box components");
    println!("  lists     - Test list components");
    println!("  messages  - Test message components");
    println!("  tables    - Test table components");
    println!("  progress  - Test progress components");
    println!("  symbols   - Test symbol components");
    println!("  styles    - Test style components");
    println!("  inputs    - Test input components (interactive)");
    println!("  help      - Test help formatting components");
    println!("  all       - Test all components");
}

fn test_all() {
    println!("{}", ui::section_header("Testing All UI Components"));

    test_boxes();
    wait_for_key();

    test_lists();
    wait_for_key();

    test_messages();
    wait_for_key();

    test_tables();
    wait_for_key();

    test_symbols();
    wait_for_key();

    test_styles();
    wait_for_key();

    test_help();
    wait_for_key();

    test_progress();
    wait_for_key();

    println!("Interactive input tests run separately due to their nature.");
    println!("Run 'cargo run --bin test-ui inputs' to test input components.");
}

fn test_boxes() {
    println!("{}", ui::section_header("Box Components"));

    println!("{}", ui::info_box("This is an info box with some content.\nIt can contain multiple lines of text and will automatically wrap if the content is too long for the terminal width."));
    println!();

    println!("{}", ui::success_box("This is a success box. Operation completed successfully!"));
    println!();

    println!("{}", ui::warning_box("This is a warning box. Please proceed with caution."));
    println!();

    println!("{}", ui::error_box("This is an error box. Something went wrong!"));
    println!();

    println!("{}", ui::help_box("This is a help box with instructions.\n\n1. First do this\n2. Then do that\n3. Finally, complete the task"));
    println!();

    println!("{}", ui::plain_box(Some("Custom Title"), "This is a plain box with a custom title."));
    println!();
}

fn test_lists() {
    println!("{}", ui::section_header("List Components"));

    // Bullet list
    let items = vec![
        "First item in the list",
        "Second item with a bit more text that might wrap around depending on your terminal width",
        "Third item",
        "Fourth item with\nmultiple\nlines",
    ];

    println!("Bullet List:");
    println!("{}", ui::bullet_list(items.clone()));
    println!();

    // Numbered list
    println!("Numbered List:");
    println!("{}", ui::numbered_list(items.clone()));
    println!();

    // Separated list
    println!("Separated List:");
    println!("{}", ui::separated_list(items.clone()));
    println!();

    // Custom list with builder
    println!("Custom List using ListBuilder:");
    let custom_list = ui::ListBuilder::new().bullets().indent(4).items(items).build();
    println!("{}", custom_list);
    println!();
}

fn test_messages() {
    println!("{}", ui::section_header("Message Components"));

    println!("{}", ui::info("This is an info message"));
    println!("{}", ui::success("This is a success message"));
    println!("{}", ui::warning("This is a warning message"));
    println!("{}", ui::error("This is an error message"));
    println!();

    println!("{}", ui::primary("This is styled with primary color"));
    println!("{}", ui::secondary("This is styled with secondary color"));
    println!("{}", ui::highlight("This is styled with highlight color"));
    println!("{}", ui::muted("This is styled with muted color"));
    println!();

    println!("{}", ui::command_example("workspace info --verbose"));
    println!("{}", ui::file_path("/path/to/some/file.js"));
    println!("{}", ui::key_value("Package", "my-package@1.0.0"));
    println!();
}

fn test_tables() {
    println!("{}", ui::section_header("Table Components"));

    // Simple table
    let headers = vec!["Name".to_string(), "Version".to_string(), "Status".to_string()];
    let rows = vec![
        vec!["package-a", "1.0.0", "Up to date"],
        vec!["package-b", "2.1.0", "Needs update"],
        vec!["package-c", "0.5.0", "Deprecated"],
        vec!["very-long-package-name", "1.0.0-beta.1", "Under development"],
    ];

    println!("{}", ui::create_table(headers, rows));
    println!();

    // Key-value table
    let items = vec![
        ("Name", "My Awesome Project"),
        ("Version", "1.0.0"),
        ("Author", "John Doe <john@example.com>"),
        ("License", "MIT"),
        ("Dependencies", "15"),
    ];

    println!("{}", ui::key_value_table(items));
    println!();
}

fn test_progress() {
    println!("{}", ui::section_header("Progress Components"));

    // Test basic spinner
    println!("Starting a spinner (will run for 3 seconds)...");
    let sp = ui::spinner("Processing something...");
    std::thread::sleep(Duration::from_secs(3));
    sp.finish_with_message("Processing complete!");
    println!();

    // Test standard progress bar
    println!("Showing a progress bar...");
    let pb = ui::progress_bar(100);
    for i in 0..100 {
        pb.set_position(i + 1);
        std::thread::sleep(Duration::from_millis(20));
    }
    pb.finish_with_message("Done!");
    println!();

    // Test download progress bar
    println!("Showing a download progress bar...");
    let dpb = ui::download_progress_bar(1024);
    for i in 0..20 {
        dpb.set_position(i * 51 + 1);
        std::thread::sleep(Duration::from_millis(100));
    }
    dpb.finish_with_message("Download complete!");
    println!();

    // Test custom progress bar
    println!("Showing a custom progress bar...");
    let cpb = ui::custom_progress_bar(
        100,
        "{spinner:.green} [{elapsed_precise}] Custom: {bar:40.cyan/blue} {pos}/{len}",
    );
    for i in 0..100 {
        cpb.set_position(i + 1);
        std::thread::sleep(Duration::from_millis(20));
    }
    cpb.finish_with_message("Custom progress complete!");
    println!();

    // Test progress_vec
    println!("Testing progress_vec...");
    let vec_data: Vec<i32> = (1..=20).collect();
    for _item in ui::progress_vec(vec_data, "Processing vector items") {
        std::thread::sleep(Duration::from_millis(100));
    }
    println!();

    // Test multi-progress
    println!("Testing multi-progress...");
    let mp = ui::multi_progress();

    let pb1 = mp.add(ui::progress_bar(50));
    pb1.set_message("Task 1".to_string());

    let pb2 = mp.add(ui::progress_bar(100));
    pb2.set_message("Task 2".to_string());

    let pb3 = mp.add(ui::progress_bar(75));
    pb3.set_message("Task 3".to_string());

    // Simulate progress on multiple bars
    for i in 0..50 {
        pb1.set_position(i + 1);

        if i % 2 == 0 {
            pb2.set_position(i * 2 + 1);
        }

        if i % 3 == 0 {
            pb3.set_position(i * 1.5 as u64 + 1);
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    pb1.finish_with_message("Task 1 complete");

    for i in 50..100 {
        pb2.set_position(i + 1);

        if i % 3 == 0 && i < 75 {
            pb3.set_position(i + 1);
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    pb2.finish_with_message("Task 2 complete");
    pb3.finish_with_message("Task 3 complete");
    println!("\nMulti-progress demonstration complete!\n");

    // Test staged progress
    println!("Demonstrating staged progress...");
    let mut staged = ui::StagedProgress::new(3, "Building Project");

    staged.start_stage("Compiling", 50);
    for _i in 0..50 {
        staged.inc(1);
        std::thread::sleep(Duration::from_millis(20));
    }
    staged.complete_stage();

    staged.start_stage("Testing", 20);
    for _i in 0..20 {
        staged.inc(1);
        std::thread::sleep(Duration::from_millis(30));
    }
    staged.complete_stage();

    staged.start_stage("Packaging", 10);
    for i in 0..10 {
        staged.inc(1);
        staged.set_stage_message(&format!("Processing item {}/10", i + 1));
        std::thread::sleep(Duration::from_millis(50));
    }
    staged.complete_stage();

    staged.finish();
    println!("\nStaged progress demonstration complete!\n");
}

fn test_symbols() {
    println!("{}", ui::section_header("Symbol Components"));

    println!("Info symbol: {}", ui::Symbol::info());
    println!("Success symbol: {}", ui::Symbol::success());
    println!("Warning symbol: {}", ui::Symbol::warning());
    println!("Error symbol: {}", ui::Symbol::error());
    println!("Pending symbol: {}", ui::Symbol::pending());
    println!("Running symbol: {}", ui::Symbol::running());
    println!("Bullet symbol: {}", ui::Symbol::bullet());
    println!("Arrow right symbol: {}", ui::Symbol::arrow_right());
    println!("Check symbol: {}", ui::Symbol::check());
    println!("Cross symbol: {}", ui::Symbol::cross());
    println!("Star symbol: {}", ui::Symbol::star());
    println!("Dot symbol: {}", ui::Symbol::dot());
    println!();
}

fn test_styles() {
    println!("{}", ui::section_header("Style Components"));

    println!("{}", ui::info_style("Info styled text"));
    println!("{}", ui::success_style("Success styled text"));
    println!("{}", ui::warning_style("Warning styled text"));
    println!("{}", ui::error_style("Error styled text"));
    println!("{}", ui::primary_style("Primary styled text"));
    println!("{}", ui::secondary_style("Secondary styled text"));
    println!("{}", ui::muted_style("Muted styled text"));
    println!("{}", ui::highlight_style("Highlight styled text"));
    println!("{}", ui::dim_style("Dim styled text"));
    println!();

    // Test theme changing
    println!("Testing theme changes:");
    println!("Default theme:");
    test_theme_sample();

    println!("\nChanging to dark theme:");
    ui::init_with_theme("dark").unwrap();
    test_theme_sample();

    println!("\nChanging to light theme:");
    ui::init_with_theme("light").unwrap();
    test_theme_sample();

    println!("\nRestoring default theme:");
    ui::init_with_theme("default").unwrap();
    test_theme_sample();
    println!();
}

fn test_theme_sample() {
    println!("{}", ui::info_style("Info"));
    println!("{}", ui::success_style("Success"));
    println!("{}", ui::warning_style("Warning"));
    println!("{}", ui::error_style("Error"));
    println!("{}", ui::primary_style("Primary"));
    println!("{}", ui::secondary_style("Secondary"));
}

fn test_inputs() {
    println!("{}", ui::section_header("Input Components (Interactive)"));

    // Test basic text input
    match ui::text("Enter some text: ") {
        Ok(input) => println!("You entered: {}", input),
        Err(e) => println!("Error: {}", e),
    }

    // Test text with default
    match ui::text_with_default("Enter text (or press Enter for default): ", "default value") {
        Ok(input) => println!("You entered: {}", input),
        Err(e) => println!("Error: {}", e),
    }

    // Test password input
    match ui::password("Enter a password: ") {
        Ok(input) => println!("Password entered (length: {})", input.len()),
        Err(e) => println!("Error: {}", e),
    }

    // Test confirmation
    match ui::confirm("Proceed with operation?", true) {
        Ok(confirmed) => println!("You chose: {}", if confirmed { "Yes" } else { "No" }),
        Err(e) => println!("Error: {}", e),
    }

    // Test selection
    let options = vec!["Option 1", "Option 2", "Option 3", "Option 4"];
    match ui::select("Choose an option:", &options) {
        Ok(index) => println!("You selected: {} (index: {})", options[index], index),
        Err(e) => println!("Error: {}", e),
    }

    // Test multi-selection
    match ui::multi_select("Select multiple options:", &options) {
        Ok(indices) => {
            println!("You selected:");
            for &i in &indices {
                println!("- {} (index: {})", options[i], i);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test input prompt with validation
    let prompt =
        ui::InputPrompt::new("Enter a number between 1-10: ").validate(|input| {
            match input.parse::<i32>() {
                Ok(n) if (1..=10).contains(&n) => Ok(()),
                Ok(_) => Err("Number must be between 1 and 10".to_string()),
                Err(_) => Err("Please enter a valid number".to_string()),
            }
        });

    match prompt.interact() {
        Ok(input) => println!("Valid input: {}", input),
        Err(e) => println!("Error: {}", e),
    }
}

fn test_help() {
    println!("{}", ui::section_header("Help Components"));

    // Test format_command
    println!("Formatted commands:");
    println!("{}", ui::format_command("init", "Initialize a new workspace"));
    println!("{}", ui::format_command("build", "Build all packages"));
    println!();

    // Test usage_example
    println!("Usage example:");
    println!("{}", ui::usage_example("workspace init --name my-project"));
    println!();

    // Test format_option
    println!("Formatted options:");
    println!("{}", ui::format_option("--verbose", "Enable verbose output"));
    println!("{}", ui::format_option("--config <FILE>", "Specify config file"));
    println!();

    // Test command_help
    println!("Complete command help:");
    let options = vec![
        ("--all", "Process all packages"),
        ("--verbose", "Enable verbose output"),
        ("--json", "Output in JSON format"),
    ];

    println!(
        "{}",
        ui::command_help(
            "build",
            "Build all or specific packages in the workspace",
            "workspace build [options] [packages...]",
            &options
        )
    );
    println!();

    // Test command_section
    println!("Command section:");
    let commands = vec![
        ("init", "Initialize a new workspace"),
        ("add", "Add a new package to the workspace"),
        ("remove", "Remove a package from the workspace"),
        ("build", "Build all or specific packages"),
        ("test", "Run tests for all or specific packages"),
    ];

    println!("{}", ui::command_section("WORKSPACE COMMANDS", &commands));
    println!();

    // Test help_box
    println!(
        "{}",
        ui::help_box(
            "workspace init     Initialize a workspace\n\
         workspace add      Add a package\n\
         workspace build    Build packages\n\
         workspace test     Run tests\n\
         workspace publish  Publish packages"
        )
    );
}

fn wait_for_key() {
    println!("\nPress any key to continue...");
    let term = Term::stdout();
    let _ = term.read_key();
    println!("\n");
}
