# Testing the Daemon Implementation

You're right to test what we've built before moving on. Here's a comprehensive testing strategy for our daemon implementation:

## 1. Build the Project

First, make sure everything compiles:

```bash
cargo build
```

## 2. Manual CLI Testing

### Basic Lifecycle Tests

Test the daemon's start, status, and stop operations:

```bash
# Start the daemon
cargo run -- daemon --start

# Check the status
cargo run -- daemon --status

# Stop the daemon
cargo run -- daemon --stop

# Restart the daemon
cargo run -- daemon --restart

# Run in foreground (for debugging)
cargo run -- daemon --start --foreground
```

### Status Output Verification

When running the status command, verify:
- The daemon correctly reports as running/not running
- The PID is shown when running
- Uptime is reported correctly
- Socket path is correctly displayed

## 3. File System Monitoring Test

You can test the file system monitoring by:

1. Starting the daemon
2. Creating a test repository for it to monitor
3. Making file changes and seeing if events are captured

```bash
# Start the daemon in foreground mode (to see logs)
cargo run -- daemon --start --foreground

# In another terminal, create a test repo
mkdir -p ~/test-repo
cd ~/test-repo
git init

# Add the repository to the daemon
# (You'll need to implement this part of the CLI)
cargo run -- daemon add-repo ~/test-repo

# Make some file changes
echo "test content" > test.txt
git add test.txt
git commit -m "Add test file"

# Check if the daemon detected the changes
# This should appear in the daemon's logs or via a status command
```

## 4. IPC Testing

We can test IPC communication by:

1. Starting the daemon
2. Using the client to send messages
3. Verifying responses

For now, the main IPC test is the status command, which should:
- Successfully communicate with the daemon
- Return proper status information
- Handle the daemon being offline gracefully

## 5. Logging and Error Handling

Test error scenarios:

```bash
# Start the daemon twice (should report it's already running)
cargo run -- daemon --start
cargo run -- daemon --start

# Stop when not running
cargo run -- daemon --stop
cargo run -- daemon --stop

# Try to create invalid repositories
# (Once we implement this functionality)
```

## 6. Create Simple Integration Tests

For more thorough testing, create simple integration tests in the `tests` directory:

```rust
// tests/daemon_tests.rs
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_daemon_lifecycle() {
    // Start daemon
    let start_output = Command::new("cargo")
        .args(&["run", "--", "daemon", "--start"])
        .output()
        .expect("Failed to start daemon");
    
    assert!(start_output.status.success());
    
    // Allow daemon to initialize
    thread::sleep(Duration::from_secs(1));
    
    // Check status
    let status_output = Command::new("cargo")
        .args(&["run", "--", "daemon", "--status"])
        .output()
        .expect("Failed to get daemon status");
    
    assert!(status_output.status.success());
    assert!(String::from_utf8_lossy(&status_output.stdout).contains("Running"));
    
    // Stop daemon
    let stop_output = Command::new("cargo")
        .args(&["run", "--", "daemon", "--stop"])
        .output()
        .expect("Failed to stop daemon");
    
    assert!(stop_output.status.success());
    
    // Verify it's stopped
    thread::sleep(Duration::from_secs(1));
    
    let final_status = Command::new("cargo")
        .args(&["run", "--", "daemon", "--status"])
        .output()
        .expect("Failed to get final daemon status");
    
    assert!(String::from_utf8_lossy(&final_status.stdout).contains("not running"));
}
```

## 7. Known Limitations for Testing

Be aware of these testing challenges:

1. **Socket file cleanup**: If tests terminate abnormally, socket files may remain - check for and delete stale socket files at `/path/to/socket`

2. **Process management**: Killing the test process might leave daemon processes running - use system tools like `ps` to find and kill stray processes

3. **Permission issues**: Unix socket operations might have permission issues in some environments

4. **Timing issues**: Some operations need time to complete - add appropriate sleeps in your tests

This testing strategy will help validate that our daemon implementation is working correctly before we proceed to the repository management step.

## Next Steps:

According to our implementation plan, the next phase would be:

**Phase 2, Step 5: Repository Management**
- Create repository registry
- Implement repository registration/unregistration
- Add repository metadata storage
- Integrate with configuration system