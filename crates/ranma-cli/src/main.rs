use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;

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
    Start(StartCmd),
    Add(AddCmd),
    Set(SetCmd),
    Remove(RemoveCmd),
    Query(QueryCmd),
    Displays(DisplaysCmd),
}

/// start the ranma server
#[derive(FromArgs)]
#[argh(subcommand, name = "start")]
struct StartCmd {
    /// path to ranma-server binary
    #[argh(option)]
    server_path: Option<String>,

    /// path to init script (overrides ~/.config/ranma/init)
    #[argh(option, long = "init")]
    init_script: Option<String>,
}

/// add a node to the bar
#[derive(FromArgs)]
#[argh(subcommand, name = "add")]
struct AddCmd {
    /// node name
    #[argh(positional)]
    name: String,

    /// node type: item (default) or container
    #[argh(option, long = "type")]
    node_type: Option<String>,

    /// parent container name
    #[argh(option)]
    parent: Option<String>,

    /// text label
    #[argh(option)]
    label: Option<String>,

    /// label color (hex, e.g. #FFCC00)
    #[argh(option)]
    label_color: Option<String>,

    /// SF Symbol icon name
    #[argh(option)]
    icon: Option<String>,

    /// icon color (hex, e.g. #FF0000)
    #[argh(option)]
    icon_color: Option<String>,

    /// background color (hex)
    #[argh(option)]
    background_color: Option<String>,

    /// border color (hex)
    #[argh(option)]
    border_color: Option<String>,

    /// border width
    #[argh(option)]
    border_width: Option<f32>,

    /// corner radius
    #[argh(option)]
    corner_radius: Option<f32>,

    /// padding left
    #[argh(option)]
    padding_left: Option<f32>,

    /// padding right
    #[argh(option)]
    padding_right: Option<f32>,

    /// shadow color (hex)
    #[argh(option)]
    shadow_color: Option<String>,

    /// shadow blur radius
    #[argh(option)]
    shadow_radius: Option<f32>,

    /// fixed width
    #[argh(option)]
    width: Option<f32>,

    /// container height
    #[argh(option)]
    height: Option<f32>,

    /// item spacing within container
    #[argh(option)]
    gap: Option<f32>,

    /// font size (default 12)
    #[argh(option)]
    font_size: Option<f32>,

    /// font weight (light, regular, medium, semibold, bold)
    #[argh(option)]
    font_weight: Option<String>,

    /// font family (e.g. "SF Mono")
    #[argh(option)]
    font_family: Option<String>,

    /// notch alignment: left or right (only effective on notched displays)
    #[argh(option)]
    notch_align: Option<String>,

    /// sort position
    #[argh(option)]
    position: Option<i32>,

    /// target display ID
    #[argh(option)]
    display: Option<u32>,
}

/// update node properties
#[derive(FromArgs)]
#[argh(subcommand, name = "set")]
struct SetCmd {
    /// node name
    #[argh(positional)]
    name: String,

    /// parent container name
    #[argh(option)]
    parent: Option<String>,

    /// text label
    #[argh(option)]
    label: Option<String>,

    /// label color (hex)
    #[argh(option)]
    label_color: Option<String>,

    /// SF Symbol icon name
    #[argh(option)]
    icon: Option<String>,

    /// icon color (hex)
    #[argh(option)]
    icon_color: Option<String>,

    /// background color (hex)
    #[argh(option)]
    background_color: Option<String>,

    /// border color (hex)
    #[argh(option)]
    border_color: Option<String>,

    /// border width
    #[argh(option)]
    border_width: Option<f32>,

    /// corner radius
    #[argh(option)]
    corner_radius: Option<f32>,

    /// padding left
    #[argh(option)]
    padding_left: Option<f32>,

    /// padding right
    #[argh(option)]
    padding_right: Option<f32>,

    /// shadow color (hex)
    #[argh(option)]
    shadow_color: Option<String>,

    /// shadow blur radius
    #[argh(option)]
    shadow_radius: Option<f32>,

    /// fixed width
    #[argh(option)]
    width: Option<f32>,

    /// container height
    #[argh(option)]
    height: Option<f32>,

    /// item spacing within container
    #[argh(option)]
    gap: Option<f32>,

    /// font size (default 12)
    #[argh(option)]
    font_size: Option<f32>,

    /// font weight (light, regular, medium, semibold, bold)
    #[argh(option)]
    font_weight: Option<String>,

    /// font family (e.g. "SF Mono")
    #[argh(option)]
    font_family: Option<String>,

    /// notch alignment: left or right (only effective on notched displays)
    #[argh(option)]
    notch_align: Option<String>,

    /// sort position
    #[argh(option)]
    position: Option<i32>,

    /// move to display ID
    #[argh(option)]
    display: Option<u32>,
}

/// remove a node
#[derive(FromArgs)]
#[argh(subcommand, name = "remove")]
struct RemoveCmd {
    /// node name
    #[argh(positional)]
    name: String,
}

/// query nodes
#[derive(FromArgs)]
#[argh(subcommand, name = "query")]
struct QueryCmd {
    /// node name (optional, query all if omitted)
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

    if let Command::Start(cmd) = args.command {
        exec_server(cmd);
        return;
    }

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

fn exec_server(cmd: StartCmd) {
    let server_path = cmd.server_path.unwrap_or_else(|| {
        let exe = std::env::current_exe().expect("cannot determine executable path");
        let dir = exe.parent().expect("cannot determine executable directory");
        dir.join("ranma-server").to_string_lossy().into_owned()
    });

    let mut command = std::process::Command::new(&server_path);
    if let Some(init) = cmd.init_script {
        command.env("RANMA_INIT", init);
    }
    let err = command.exec();
    eprintln!("error: failed to exec {}: {}", server_path, err);
    std::process::exit(1);
}

fn build_command(cmd: Command) -> Value {
    match cmd {
        Command::Start(_) => unreachable!(),
        Command::Add(c) => {
            let mut obj = json!({
                "command": "add",
                "name": c.name,
            });
            if let Some(v) = c.node_type { obj["node_type"] = json!(v); }
            if let Some(v) = c.parent { obj["parent"] = json!(v); }
            if let Some(v) = c.label { obj["label"] = json!(v); }
            if let Some(v) = c.label_color { obj["label_color"] = json!(v); }
            if let Some(v) = c.icon { obj["icon"] = json!(v); }
            if let Some(v) = c.icon_color { obj["icon_color"] = json!(v); }
            if let Some(v) = c.background_color { obj["background_color"] = json!(v); }
            if let Some(v) = c.border_color { obj["border_color"] = json!(v); }
            if let Some(v) = c.border_width { obj["border_width"] = json!(v); }
            if let Some(v) = c.corner_radius { obj["corner_radius"] = json!(v); }
            if let Some(v) = c.padding_left { obj["padding_left"] = json!(v); }
            if let Some(v) = c.padding_right { obj["padding_right"] = json!(v); }
            if let Some(v) = c.shadow_color { obj["shadow_color"] = json!(v); }
            if let Some(v) = c.shadow_radius { obj["shadow_radius"] = json!(v); }
            if let Some(v) = c.width { obj["width"] = json!(v); }
            if let Some(v) = c.height { obj["height"] = json!(v); }
            if let Some(v) = c.gap { obj["gap"] = json!(v); }
            if let Some(v) = c.font_size { obj["font_size"] = json!(v); }
            if let Some(v) = c.font_weight { obj["font_weight"] = json!(v); }
            if let Some(v) = c.font_family { obj["font_family"] = json!(v); }
            if let Some(v) = c.notch_align { obj["notch_align"] = json!(v); }
            if let Some(v) = c.position { obj["position"] = json!(v); }
            if let Some(v) = c.display { obj["display"] = json!(v); }
            obj
        }
        Command::Set(c) => {
            let mut properties: HashMap<String, String> = HashMap::new();
            if let Some(v) = c.parent { properties.insert("parent".into(), v); }
            if let Some(v) = c.label { properties.insert("label".into(), v); }
            if let Some(v) = c.label_color { properties.insert("label_color".into(), v); }
            if let Some(v) = c.icon { properties.insert("icon".into(), v); }
            if let Some(v) = c.icon_color { properties.insert("icon_color".into(), v); }
            if let Some(v) = c.background_color { properties.insert("background_color".into(), v); }
            if let Some(v) = c.border_color { properties.insert("border_color".into(), v); }
            if let Some(v) = c.border_width { properties.insert("border_width".into(), v.to_string()); }
            if let Some(v) = c.corner_radius { properties.insert("corner_radius".into(), v.to_string()); }
            if let Some(v) = c.padding_left { properties.insert("padding_left".into(), v.to_string()); }
            if let Some(v) = c.padding_right { properties.insert("padding_right".into(), v.to_string()); }
            if let Some(v) = c.shadow_color { properties.insert("shadow_color".into(), v); }
            if let Some(v) = c.shadow_radius { properties.insert("shadow_radius".into(), v.to_string()); }
            if let Some(v) = c.width { properties.insert("width".into(), v.to_string()); }
            if let Some(v) = c.height { properties.insert("height".into(), v.to_string()); }
            if let Some(v) = c.gap { properties.insert("gap".into(), v.to_string()); }
            if let Some(v) = c.font_size { properties.insert("font_size".into(), v.to_string()); }
            if let Some(v) = c.font_weight { properties.insert("font_weight".into(), v); }
            if let Some(v) = c.font_family { properties.insert("font_family".into(), v); }
            if let Some(v) = c.notch_align { properties.insert("notch_align".into(), v); }
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
