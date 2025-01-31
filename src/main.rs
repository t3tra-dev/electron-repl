use clap::{Arg, Command};
use reqwest;
use rustyline::{error::ReadlineError, Editor};
use serde::Deserialize;
use std::process::{Child, Stdio};
use std::time::Duration;
use tungstenite::connect;

#[derive(Deserialize)]
struct DevToolsTarget {
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: Option<String>,
}

async fn get_debugger_url(port: u16) -> Option<String> {
    let url = format!("http://127.0.0.1:{}/json", port);
    for _ in 0..20 {
        match reqwest::get(&url).await {
            Ok(response) => match response.json::<Vec<DevToolsTarget>>().await {
                Ok(json) => {
                    if let Some(debugger_url) = json
                        .into_iter()
                        .find_map(|target| target.web_socket_debugger_url)
                    {
                        return Some(debugger_url);
                    }
                }
                Err(err) => eprintln!("[!] JSON Parse Failed: {}", err),
            },
            Err(err) => eprintln!("[!] HTTP Request Failed: {}", err),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    None
}

fn start_electron_app(app_name: &str, port: u16) -> Option<Child> {
    if cfg!(target_os = "windows") {
        eprintln!("[!] Windows is not supported yet.");
        return None;
    }

    let possible_paths = if cfg!(target_os = "macos") {
        vec![
            format!("/Applications/{}.app/Contents/MacOS/{}", app_name, app_name),
            format!("/Applications/{}.app/Contents/MacOS/{}", app_name.to_lowercase(), app_name.to_lowercase()),
        ]
    } else {
        vec![
            format!("/usr/bin/{}", app_name.to_lowercase()),
            format!("/usr/local/bin/{}", app_name.to_lowercase()),
            format!("/opt/{}/{}", app_name.to_lowercase(), app_name.to_lowercase()),
            format!("/opt/{}/bin/{}", app_name.to_lowercase(), app_name.to_lowercase()),
            format!("/snap/{}/current/{}", app_name.to_lowercase(), app_name.to_lowercase()),
            format!("{}/Applications/{}.AppImage", std::env::var("HOME").unwrap_or_default(), app_name),
        ]
    };

    for app_path in possible_paths {
        if std::path::Path::new(&app_path).exists() {
            if let Ok(child) = std::process::Command::new(&app_path)
                .arg(format!("--remote-debugging-port={}", port))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                return Some(child);
            }
        }
    }

    eprintln!("[!] Could not find {} in any of the standard locations", app_name);
    None
}

async fn repl(debugger_url: String, app_name: &str) {
    let (mut socket, _) = match connect(&debugger_url) {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("[!] WebSocket Connection Failed: {}", err);
            return;
        }
    };

    let mut rl = Editor::<(), rustyline::history::FileHistory>::new().unwrap();

    println!("[*] Electron REPL Activated");

    loop {
        match rl.readline(&format!("{}> ", app_name)) {
            Ok(input) => {
                let input = input.trim();
                if input == "exit" {
                    break;
                }

                rl.add_history_entry(input).unwrap();

                let json = serde_json::json!({
                    "id": 1,
                    "method": "Runtime.evaluate",
                    "params": {
                        "expression": input,
                        "contextId": 1,
                        "returnByValue": true,
                        "generatePreview": true
                    }
                });

                if let Err(err) = socket.send(tungstenite::Message::Text(json.to_string().into())) {
                    eprintln!("[!] WebSocket transmission failure: {}", err);
                    continue;
                }

                if let Ok(message) = socket.read() {
                    if let tungstenite::Message::Text(response) = message {
                        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();

                        if let Some(error_details) =
                            parsed["result"]["exceptionDetails"].as_object()
                        {
                            if let Some(exception) = error_details.get("exception") {
                                if let Some(description) = exception["description"].as_str() {
                                    println!("\x1b[31m{}\x1b[0m", description);
                                    continue;
                                }
                            }
                        }

                        if let Some(result) = parsed["result"]["result"].as_object() {
                            let output = if let Some(description) = result.get("description") {
                                description.as_str().unwrap_or("(invalid description)")
                            } else if let Some(value) = result.get("value") {
                                match value {
                                    serde_json::Value::String(s) => s,
                                    serde_json::Value::Number(n) => &n.to_string(),
                                    serde_json::Value::Bool(b) => &b.to_string(),
                                    _ => "(complex value)",
                                }
                            } else if let Some(type_str) = result.get("type") {
                                type_str.as_str().unwrap_or("(unknown type)")
                            } else {
                                "(no output)"
                            };

                            println!("\x1b[32m<- {}\x1b[0m", output);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => eprintln!("[!] Input error: {}", err),
        }
    }
}

#[tokio::main]
async fn main() {
    let matches = Command::new("electron-repl")
        .about("REPL for Electron apps.")
        .arg(
            Arg::new("app")
                .required(true)
                .help("Target Electron app name"),
        )
        .arg(
            Arg::new("port")
                .required(false)
                .default_value("9222")
                .help("Port number for DevTools"),
        )
        .get_matches();

    let app_name = matches.get_one::<String>("app").unwrap();
    let port: u16 = matches.get_one::<String>("port").unwrap().parse().unwrap();

    let child = start_electron_app(app_name, port);
    if child.is_none() {
        eprintln!("[!] Failed to start app: {}", app_name);
        return;
    }

    let debugger_url = get_debugger_url(port).await;
    if let Some(debugger_url) = debugger_url {
        repl(debugger_url, app_name).await;
    } else {
        eprintln!("[!] Failed to retrieve WebSocket");
    }
}
