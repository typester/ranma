use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use argh::FromArgs;
use serde_json::{json, Value};

/// ranma status bar controller
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    Add(AddCmd),
    Set(SetCmd),
    Remove(RemoveCmd),
    Query(QueryCmd),
    Displays(DisplaysCmd),
}

/// add an item to the bar
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
struct AddCmd {
    /// item name
    #[argh(positional)]
    name: String,

    /// text label
    #[argh(option)]
    label: Option<String>,

    /// SF Symbol icon name
    #[argh(option)]
    icon: Option<String>,

    /// icon color (hex, e.g. #FF0000)
    #[argh(option)]
    icon_color: Option<String>,

    /// background color (hex)
    #[argh(option)]
    background_color: Option<String>,

    /// sort position
    #[argh(option)]
    position: Option<i32>,

    /// target display ID
    #[argh(option)]
    display: Option<u32>,
}

/// update item properties
#[derive(FromArgs)]
#[argh(subcommand, name = "set")]
struct SetCmd {
    /// item name
    #[argh(positional)]
    name: String,

    /// text label
    #[argh(option)]
    label: Option<String>,

    /// SF Symbol icon name
    #[argh(option)]
    icon: Option<String>,

    /// icon color (hex)
    #[argh(option)]
    icon_color: Option<String>,

    /// background color (hex)
    #[argh(option)]
    background_color: Option<String>,

    /// sort position
    #[argh(option)]
    position: Option<i32>,

    /// move to display ID
    #[argh(option)]
    display: Option<u32>,
}

/// remove an item
#[derive(FromArgs)]
#[argh(subcommand, name = "remove")]
struct RemoveCmd {
    /// item name
    #[argh(positional)]
    name: String,
}

/// query items
#[derive(FromArgs)]
#[argh(subcommand, name = "query")]
struct QueryCmd {
    /// item name (optional, query all if omitted)
    #[argh(positional)]
    name: Option<String>,

    /// filter by display ID
    #[argh(option)]
    display: Option<u32>,
}

/// list connected displays
#[derive(FromArgs)]
#[argh(subcommand, name = "displays")]
struct DisplaysCmd {}

fn main() {
    let args: Args = argh::from_env();

    let command = build_command(args.command);

    let socket_path = default_socket_path();
    match send_command(&socket_path, &command) {
        Ok(response) => println!("{response}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn build_command(cmd: Command) -> Value {
    match cmd {
        Command::Add(c) => {
            let mut obj = json!({
                "command": "add",
                "name": c.name,
            });
            if let Some(v) = c.label { obj["label"] = json!(v); }
            if let Some(v) = c.icon { obj["icon"] = json!(v); }
            if let Some(v) = c.icon_color { obj["icon_color"] = json!(v); }
            if let Some(v) = c.background_color { obj["background_color"] = json!(v); }
            if let Some(v) = c.position { obj["position"] = json!(v); }
            if let Some(v) = c.display { obj["display"] = json!(v); }
            obj
        }
        Command::Set(c) => {
            let mut properties: HashMap<String, String> = HashMap::new();
            if let Some(v) = c.label { properties.insert("label".into(), v); }
            if let Some(v) = c.icon { properties.insert("icon".into(), v); }
            if let Some(v) = c.icon_color { properties.insert("icon_color".into(), v); }
            if let Some(v) = c.background_color { properties.insert("background_color".into(), v); }
            if let Some(v) = c.position { properties.insert("position".into(), v.to_string()); }
            if let Some(v) = c.display { properties.insert("display".into(), v.to_string()); }
            json!({
                "command": "set",
                "name": c.name,
                "properties": properties,
            })
        }
        Command::Remove(c) => json!({ "command": "remove", "name": c.name }),
        Command::Query(c) => json!({ "command": "query", "name": c.name, "display": c.display }),
        Command::Displays(_) => json!({ "command": "displays" }),
    }
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
