use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdout, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

#[test]
fn stdio_server_lists_and_calls_nju_cli_tool() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nju-cli-mcp"))
        .env("NJU_CLI_BIN", "printf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start nju-cli-mcp");
    let mut stdin = child.stdin.take().expect("child stdin");
    let stdout = child.stdout.take().expect("child stdout");
    let mut stdout = BufReader::new(stdout);

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {
                    "name": "nju-cli-mcp-test",
                    "version": "0.0.0"
                }
            }
        }),
    );
    let initialize = receive(&mut stdout);
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["serverInfo"]["name"], "nju-cli-mcp");

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    );

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    );
    let tools = receive(&mut stdout);
    let tool = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .find(|tool| tool["name"] == "nju-cli")
        .expect("nju-cli tool");
    assert!(tool["inputSchema"]["properties"]["args"].is_object());
    assert!(tool["outputSchema"]["properties"]["stdout"].is_object());

    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "nju-cli",
                "arguments": {
                    "args": ["%s:%s", "a", "b"]
                }
            }
        }),
    );
    let call = receive(&mut stdout);
    assert_eq!(call["id"], 3);
    assert_eq!(call["result"]["isError"], false);
    assert_eq!(call["result"]["structuredContent"]["stdout"], "a:b");

    drop(stdin);
    wait_for_exit(&mut child);
}

fn send(stdin: &mut std::process::ChildStdin, message: Value) {
    writeln!(stdin, "{message}").expect("failed to write MCP message");
    stdin.flush().expect("failed to flush MCP message");
}

fn receive(stdout: &mut BufReader<ChildStdout>) -> Value {
    let mut line = String::new();
    stdout
        .read_line(&mut line)
        .expect("failed to read MCP message");
    assert!(!line.is_empty(), "MCP server closed stdout");

    serde_json::from_str(&line).expect("invalid MCP JSON message")
}

fn wait_for_exit(child: &mut Child) {
    let started_at = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("failed to poll child status") {
            assert!(status.success(), "MCP server exited with {status}");
            return;
        }

        if started_at.elapsed() > Duration::from_secs(5) {
            child.kill().expect("failed to kill hung MCP server");
            let _ = child.wait();
            panic!("MCP server did not exit after stdin closed");
        }

        thread::sleep(Duration::from_millis(50));
    }
}
