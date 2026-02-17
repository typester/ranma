use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;

use argh::FromArgs;
use serde_json::{Value, json};

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
    Tree(TreeCmd),
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

    /// node type: item (default), row, column, or box
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

    /// padding top
    #[argh(option)]
    padding_top: Option<f32>,

    /// padding bottom
    #[argh(option)]
    padding_bottom: Option<f32>,

    /// padding (all sides)
    #[argh(option)]
    padding: Option<f32>,

    /// padding horizontal (left + right)
    #[argh(option)]
    padding_horizontal: Option<f32>,

    /// padding vertical (top + bottom)
    #[argh(option)]
    padding_vertical: Option<f32>,

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

    /// margin left
    #[argh(option)]
    margin_left: Option<f32>,

    /// margin right
    #[argh(option)]
    margin_right: Option<f32>,

    /// margin top
    #[argh(option)]
    margin_top: Option<f32>,

    /// margin bottom
    #[argh(option)]
    margin_bottom: Option<f32>,

    /// margin (all sides)
    #[argh(option)]
    margin: Option<f32>,

    /// margin horizontal (left + right)
    #[argh(option)]
    margin_horizontal: Option<f32>,

    /// margin vertical (top + bottom)
    #[argh(option)]
    margin_vertical: Option<f32>,

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

    /// cross-axis alignment of children: start, center, or end
    #[argh(option)]
    align_items: Option<String>,

    /// main-axis alignment of children: start, center, or end
    #[argh(option)]
    justify_content: Option<String>,

    /// background color on hover (hex)
    #[argh(option)]
    hover_background_color: Option<String>,

    /// label color on hover (hex)
    #[argh(option)]
    hover_label_color: Option<String>,

    /// icon color on hover (hex)
    #[argh(option)]
    hover_icon_color: Option<String>,

    /// shell command to run on click
    #[argh(option)]
    on_click: Option<String>,

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

    /// padding top
    #[argh(option)]
    padding_top: Option<f32>,

    /// padding bottom
    #[argh(option)]
    padding_bottom: Option<f32>,

    /// padding (all sides)
    #[argh(option)]
    padding: Option<f32>,

    /// padding horizontal (left + right)
    #[argh(option)]
    padding_horizontal: Option<f32>,

    /// padding vertical (top + bottom)
    #[argh(option)]
    padding_vertical: Option<f32>,

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

    /// margin left
    #[argh(option)]
    margin_left: Option<f32>,

    /// margin right
    #[argh(option)]
    margin_right: Option<f32>,

    /// margin top
    #[argh(option)]
    margin_top: Option<f32>,

    /// margin bottom
    #[argh(option)]
    margin_bottom: Option<f32>,

    /// margin (all sides)
    #[argh(option)]
    margin: Option<f32>,

    /// margin horizontal (left + right)
    #[argh(option)]
    margin_horizontal: Option<f32>,

    /// margin vertical (top + bottom)
    #[argh(option)]
    margin_vertical: Option<f32>,

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

    /// cross-axis alignment of children: start, center, or end
    #[argh(option)]
    align_items: Option<String>,

    /// main-axis alignment of children: start, center, or end
    #[argh(option)]
    justify_content: Option<String>,

    /// background color on hover (hex)
    #[argh(option)]
    hover_background_color: Option<String>,

    /// label color on hover (hex)
    #[argh(option)]
    hover_label_color: Option<String>,

    /// icon color on hover (hex)
    #[argh(option)]
    hover_icon_color: Option<String>,

    /// shell command to run on click
    #[argh(option)]
    on_click: Option<String>,

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

/// display node tree
#[derive(FromArgs)]
#[argh(subcommand, name = "tree")]
struct TreeCmd {
    /// filter by display ID
    #[argh(option)]
    display: Option<u32>,
}

fn main() {
    let args: Args = argh::from_env();

    if let Command::Start(cmd) = args.command {
        exec_server(cmd);
        return;
    }

    if let Command::Tree(cmd) = args.command {
        run_tree(cmd);
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
            if let Some(v) = c.node_type {
                obj["node_type"] = json!(v);
            }
            if let Some(v) = c.parent {
                obj["parent"] = json!(v);
            }
            if let Some(v) = c.label {
                obj["label"] = json!(v);
            }
            if let Some(v) = c.label_color {
                obj["label_color"] = json!(v);
            }
            if let Some(v) = c.icon {
                obj["icon"] = json!(v);
            }
            if let Some(v) = c.icon_color {
                obj["icon_color"] = json!(v);
            }
            if let Some(v) = c.background_color {
                obj["background_color"] = json!(v);
            }
            if let Some(v) = c.border_color {
                obj["border_color"] = json!(v);
            }
            if let Some(v) = c.border_width {
                obj["border_width"] = json!(v);
            }
            if let Some(v) = c.corner_radius {
                obj["corner_radius"] = json!(v);
            }
            if let Some(v) = c.padding_left {
                obj["padding_left"] = json!(v);
            }
            if let Some(v) = c.padding_right {
                obj["padding_right"] = json!(v);
            }
            if let Some(v) = c.padding_top {
                obj["padding_top"] = json!(v);
            }
            if let Some(v) = c.padding_bottom {
                obj["padding_bottom"] = json!(v);
            }
            if let Some(v) = c.padding {
                obj["padding"] = json!(v);
            }
            if let Some(v) = c.padding_horizontal {
                obj["padding_horizontal"] = json!(v);
            }
            if let Some(v) = c.padding_vertical {
                obj["padding_vertical"] = json!(v);
            }
            if let Some(v) = c.shadow_color {
                obj["shadow_color"] = json!(v);
            }
            if let Some(v) = c.shadow_radius {
                obj["shadow_radius"] = json!(v);
            }
            if let Some(v) = c.width {
                obj["width"] = json!(v);
            }
            if let Some(v) = c.height {
                obj["height"] = json!(v);
            }
            if let Some(v) = c.gap {
                obj["gap"] = json!(v);
            }
            if let Some(v) = c.margin_left {
                obj["margin_left"] = json!(v);
            }
            if let Some(v) = c.margin_right {
                obj["margin_right"] = json!(v);
            }
            if let Some(v) = c.margin_top {
                obj["margin_top"] = json!(v);
            }
            if let Some(v) = c.margin_bottom {
                obj["margin_bottom"] = json!(v);
            }
            if let Some(v) = c.margin {
                obj["margin"] = json!(v);
            }
            if let Some(v) = c.margin_horizontal {
                obj["margin_horizontal"] = json!(v);
            }
            if let Some(v) = c.margin_vertical {
                obj["margin_vertical"] = json!(v);
            }
            if let Some(v) = c.font_size {
                obj["font_size"] = json!(v);
            }
            if let Some(v) = c.font_weight {
                obj["font_weight"] = json!(v);
            }
            if let Some(v) = c.font_family {
                obj["font_family"] = json!(v);
            }
            if let Some(v) = c.notch_align {
                obj["notch_align"] = json!(v);
            }
            if let Some(v) = c.align_items {
                obj["align_items"] = json!(v);
            }
            if let Some(v) = c.justify_content {
                obj["justify_content"] = json!(v);
            }
            if let Some(v) = c.hover_background_color {
                obj["hover_background_color"] = json!(v);
            }
            if let Some(v) = c.hover_label_color {
                obj["hover_label_color"] = json!(v);
            }
            if let Some(v) = c.hover_icon_color {
                obj["hover_icon_color"] = json!(v);
            }
            if let Some(v) = c.on_click {
                obj["on_click"] = json!(v);
            }
            if let Some(v) = c.position {
                obj["position"] = json!(v);
            }
            if let Some(v) = c.display {
                obj["display"] = json!(v);
            }
            obj
        }
        Command::Set(c) => {
            let mut properties: HashMap<String, String> = HashMap::new();
            if let Some(v) = c.parent {
                properties.insert("parent".into(), v);
            }
            if let Some(v) = c.label {
                properties.insert("label".into(), v);
            }
            if let Some(v) = c.label_color {
                properties.insert("label_color".into(), v);
            }
            if let Some(v) = c.icon {
                properties.insert("icon".into(), v);
            }
            if let Some(v) = c.icon_color {
                properties.insert("icon_color".into(), v);
            }
            if let Some(v) = c.background_color {
                properties.insert("background_color".into(), v);
            }
            if let Some(v) = c.border_color {
                properties.insert("border_color".into(), v);
            }
            if let Some(v) = c.border_width {
                properties.insert("border_width".into(), v.to_string());
            }
            if let Some(v) = c.corner_radius {
                properties.insert("corner_radius".into(), v.to_string());
            }
            if let Some(v) = c.padding_left {
                properties.insert("padding_left".into(), v.to_string());
            }
            if let Some(v) = c.padding_right {
                properties.insert("padding_right".into(), v.to_string());
            }
            if let Some(v) = c.padding_top {
                properties.insert("padding_top".into(), v.to_string());
            }
            if let Some(v) = c.padding_bottom {
                properties.insert("padding_bottom".into(), v.to_string());
            }
            if let Some(v) = c.padding {
                properties.insert("padding".into(), v.to_string());
            }
            if let Some(v) = c.padding_horizontal {
                properties.insert("padding_horizontal".into(), v.to_string());
            }
            if let Some(v) = c.padding_vertical {
                properties.insert("padding_vertical".into(), v.to_string());
            }
            if let Some(v) = c.shadow_color {
                properties.insert("shadow_color".into(), v);
            }
            if let Some(v) = c.shadow_radius {
                properties.insert("shadow_radius".into(), v.to_string());
            }
            if let Some(v) = c.width {
                properties.insert("width".into(), v.to_string());
            }
            if let Some(v) = c.height {
                properties.insert("height".into(), v.to_string());
            }
            if let Some(v) = c.gap {
                properties.insert("gap".into(), v.to_string());
            }
            if let Some(v) = c.margin_left {
                properties.insert("margin_left".into(), v.to_string());
            }
            if let Some(v) = c.margin_right {
                properties.insert("margin_right".into(), v.to_string());
            }
            if let Some(v) = c.margin_top {
                properties.insert("margin_top".into(), v.to_string());
            }
            if let Some(v) = c.margin_bottom {
                properties.insert("margin_bottom".into(), v.to_string());
            }
            if let Some(v) = c.margin {
                properties.insert("margin".into(), v.to_string());
            }
            if let Some(v) = c.margin_horizontal {
                properties.insert("margin_horizontal".into(), v.to_string());
            }
            if let Some(v) = c.margin_vertical {
                properties.insert("margin_vertical".into(), v.to_string());
            }
            if let Some(v) = c.font_size {
                properties.insert("font_size".into(), v.to_string());
            }
            if let Some(v) = c.font_weight {
                properties.insert("font_weight".into(), v);
            }
            if let Some(v) = c.font_family {
                properties.insert("font_family".into(), v);
            }
            if let Some(v) = c.notch_align {
                properties.insert("notch_align".into(), v);
            }
            if let Some(v) = c.align_items {
                properties.insert("align_items".into(), v);
            }
            if let Some(v) = c.justify_content {
                properties.insert("justify_content".into(), v);
            }
            if let Some(v) = c.hover_background_color {
                properties.insert("hover_background_color".into(), v);
            }
            if let Some(v) = c.hover_label_color {
                properties.insert("hover_label_color".into(), v);
            }
            if let Some(v) = c.hover_icon_color {
                properties.insert("hover_icon_color".into(), v);
            }
            if let Some(v) = c.on_click {
                properties.insert("on_click".into(), v);
            }
            if let Some(v) = c.position {
                properties.insert("position".into(), v.to_string());
            }
            if let Some(v) = c.display {
                properties.insert("display".into(), v.to_string());
            }
            json!({
                "command": "set",
                "name": c.name,
                "properties": properties,
            })
        }
        Command::Remove(c) => json!({ "command": "remove", "name": c.name }),
        Command::Query(c) => json!({ "command": "query", "name": c.name, "display": c.display }),
        Command::Displays(_) => json!({ "command": "displays" }),
        Command::Tree(_) => unreachable!(),
    }
}

fn run_tree(cmd: TreeCmd) {
    let query = json!({ "command": "query", "name": null, "display": cmd.display });
    let socket_path = default_socket_path();
    let response = match send_command(&socket_path, &query) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let data: Value = match serde_json::from_str(&response) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: failed to parse response: {e}");
            std::process::exit(1);
        }
    };

    let nodes = match data["nodes"].as_array() {
        Some(arr) => arr,
        None => {
            eprintln!("error: unexpected response format");
            std::process::exit(1);
        }
    };

    // Group nodes by display
    let mut by_display: BTreeMap<u64, Vec<&Value>> = BTreeMap::new();
    for node in nodes {
        let display_id = node["display"].as_u64().unwrap_or(0);
        by_display.entry(display_id).or_default().push(node);
    }

    let mut first = true;
    for (display_id, display_nodes) in &by_display {
        if !first {
            println!();
        }
        first = false;

        println!("[display {display_id}]");

        // Build parent->children map
        let mut children_map: HashMap<String, Vec<&Value>> = HashMap::new();
        let mut roots: Vec<&Value> = Vec::new();

        for node in display_nodes {
            if let Some(parent) = node["parent"].as_str()
                && !parent.is_empty()
            {
                children_map
                    .entry(parent.to_string())
                    .or_default()
                    .push(node);
                continue;
            }
            roots.push(node);
        }

        // Sort by position
        let sort_by_position = |a: &&Value, b: &&Value| {
            let pa = a["position"].as_i64().unwrap_or(0);
            let pb = b["position"].as_i64().unwrap_or(0);
            pa.cmp(&pb)
        };

        roots.sort_by(sort_by_position);
        for children in children_map.values_mut() {
            children.sort_by(sort_by_position);
        }

        for root in &roots {
            print_tree_node(root, &children_map, "", true);
        }
    }
}

fn format_node_line(node: &Value) -> String {
    let name = node["name"].as_str().unwrap_or("?");
    let node_type = node["node_type"].as_str().unwrap_or("item");

    let mut line = format!("{name} ({node_type})");

    if let Some(icon) = node["icon"].as_str()
        && !icon.is_empty()
    {
        line.push_str(&format!(" icon:{icon}"));
    }
    if let Some(label) = node["label"].as_str()
        && !label.is_empty()
    {
        line.push_str(&format!(" \"{label}\""));
    }

    line
}

fn print_tree_node(
    node: &Value,
    children_map: &HashMap<String, Vec<&Value>>,
    prefix: &str,
    is_root: bool,
) {
    let line = format_node_line(node);

    if is_root {
        println!("{line}");
    }

    let name = node["name"].as_str().unwrap_or("");
    if let Some(children) = children_map.get(name) {
        let count = children.len();
        for (i, child) in children.iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let child_line = format_node_line(child);
            println!("{prefix}{connector}{child_line}");

            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            print_tree_node(child, children_map, &child_prefix, false);
        }
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
