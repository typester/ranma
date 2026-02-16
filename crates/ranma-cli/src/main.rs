use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use serde_json::{json, Value};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: ranma --add <name> [key=value ...] | --set <name> key=value ... | --remove <name> | --query [name]");
        std::process::exit(1);
    }

    let command = match parse_args(&args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let socket_path = default_socket_path();
    match send_command(&socket_path, &command) {
        Ok(response) => {
            println!("{response}");
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn parse_args(args: &[String]) -> Result<Value, String> {
    let cmd = args[0].as_str();
    let rest = &args[1..];

    match cmd {
        "--add" => {
            let name = rest.first().ok_or("--add requires a name")?;
            let props = parse_key_values(&rest[1..]);
            let mut obj = json!({ "command": "add", "name": name });
            for (k, v) in &props {
                obj[k] = json!(v);
            }
            Ok(obj)
        }
        "--set" => {
            let name = rest.first().ok_or("--set requires a name")?;
            let props = parse_key_values(&rest[1..]);
            if props.is_empty() {
                return Err("--set requires at least one key=value".into());
            }
            Ok(json!({
                "command": "set",
                "name": name,
                "properties": props,
            }))
        }
        "--remove" => {
            let name = rest.first().ok_or("--remove requires a name")?;
            Ok(json!({ "command": "remove", "name": name }))
        }
        "--query" => {
            let name = rest.first().map(|s| s.as_str());
            Ok(json!({ "command": "query", "name": name }))
        }
        _ => Err(format!("unknown command: {cmd}")),
    }
}

fn parse_key_values(args: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for arg in args {
        if let Some((k, v)) = arg.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

fn default_socket_path() -> String {
    let uid = unsafe { libc::getuid() };
    let tmp = std::env::temp_dir();
    format!("{}/ranma_{uid}.sock", tmp.display())
}

fn send_command(socket_path: &str, command: &Value) -> Result<String, String> {
    let mut stream =
        UnixStream::connect(socket_path).map_err(|e| format!("cannot connect to daemon: {e}"))?;

    let mut payload = serde_json::to_string(command).unwrap();
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write error: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read error: {e}"))?;

    Ok(response.trim_end().to_string())
}
