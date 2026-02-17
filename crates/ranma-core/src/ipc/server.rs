use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::bridge::StateChangeEvent;
use crate::state::{BarNode, NodeStyle, NodeType};
use crate::{get_displays, get_state, main_display_id, notify};

use super::protocol::{Command, DisplayDto, Response};

pub async fn run(socket_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    eprintln!("listening on {}", socket_path.display());

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let response = handle_command(&line);
        let mut out = serde_json::to_string(&response)?;
        out.push('\n');
        writer.write_all(out.as_bytes()).await?;
    }

    Ok(())
}

fn handle_command(input: &str) -> Response {
    let cmd: Command = match serde_json::from_str(input) {
        Ok(cmd) => cmd,
        Err(e) => {
            return Response::Error {
                message: format!("invalid command: {e}"),
            };
        }
    };

    match cmd {
        Command::Add {
            name,
            node_type,
            parent,
            label,
            label_color,
            icon,
            icon_color,
            background_color,
            border_color,
            border_width,
            corner_radius,
            padding_left,
            padding_right,
            padding_top,
            padding_bottom,
            shadow_color,
            shadow_radius,
            width,
            height,
            gap,
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            padding,
            padding_horizontal,
            padding_vertical,
            margin,
            margin_horizontal,
            margin_vertical,
            font_size,
            font_weight,
            font_family,
            notch_align,
            align_items,
            justify_content,
            hover_background_color,
            hover_label_color,
            hover_icon_color,
            on_click,
            position,
            display,
        } => {
            let display = display.unwrap_or_else(|| {
                if let Some(ref parent_name) = parent {
                    let state = get_state().lock();
                    state
                        .get_nodes()
                        .iter()
                        .find(|n| &n.name == parent_name)
                        .map(|n| n.display)
                        .unwrap_or_else(main_display_id)
                } else {
                    main_display_id()
                }
            });
            let nt = match node_type.as_deref() {
                Some("row") => NodeType::Row,
                Some("column") => NodeType::Column,
                Some("box") => NodeType::Box,
                _ => NodeType::Item,
            };
            let node = BarNode {
                name,
                node_type: nt,
                parent,
                label,
                label_color,
                icon,
                icon_color,
                font_size,
                font_weight,
                font_family,
                on_click,
                position: position.unwrap_or(0),
                display,
                style: NodeStyle {
                    background_color,
                    border_color,
                    border_width,
                    corner_radius,
                    padding_left: padding_left.or(padding_horizontal).or(padding),
                    padding_right: padding_right.or(padding_horizontal).or(padding),
                    padding_top: padding_top.or(padding_vertical).or(padding),
                    padding_bottom: padding_bottom.or(padding_vertical).or(padding),
                    shadow_color,
                    shadow_radius,
                    width,
                    height,
                    gap,
                    margin_left: margin_left.or(margin_horizontal).or(margin),
                    margin_right: margin_right.or(margin_horizontal).or(margin),
                    margin_top: margin_top.or(margin_vertical).or(margin),
                    margin_bottom: margin_bottom.or(margin_vertical).or(margin),
                    notch_align,
                    align_items,
                    justify_content,
                    hover_background_color,
                    hover_label_color,
                    hover_icon_color,
                },
            };
            let mut state = get_state().lock();
            match state.add_node(node.clone()) {
                Ok(()) => {
                    drop(state);
                    notify(StateChangeEvent::NodeAdded { display, node });
                    Response::Ok
                }
                Err(message) => Response::Error { message },
            }
        }
        Command::Set { name, properties } => {
            let mut state = get_state().lock();
            let old_display = state
                .get_nodes()
                .iter()
                .find(|n| n.name == name)
                .map(|n| n.display);

            match state.set_properties(&name, &properties) {
                Ok(node) => {
                    let new_display = node.display;
                    drop(state);

                    if let Some(old) = old_display {
                        if old != new_display {
                            notify(StateChangeEvent::NodeMoved {
                                old_display: old,
                                new_display,
                                node,
                            });
                        } else {
                            notify(StateChangeEvent::NodeUpdated {
                                display: new_display,
                                node,
                            });
                        }
                    }
                    Response::Ok
                }
                Err(message) => Response::Error { message },
            }
        }
        Command::Remove { name } => {
            let mut state = get_state().lock();
            match state.remove_node(&name) {
                Ok(node) => {
                    let display = node.display;
                    drop(state);
                    notify(StateChangeEvent::NodeRemoved { display, name });
                    Response::Ok
                }
                Err(message) => Response::Error { message },
            }
        }
        Command::Query { name, display } => {
            let state = get_state().lock();
            let nodes: Vec<_> = match (name, display) {
                (Some(name), _) => state
                    .get_nodes()
                    .into_iter()
                    .filter(|n| n.name == name)
                    .map(Into::into)
                    .collect(),
                (None, Some(display)) => state
                    .get_nodes_for_display(display)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                (None, None) => state.get_nodes().into_iter().map(Into::into).collect(),
            };
            Response::QueryResult { nodes }
        }
        Command::Displays => {
            let displays = get_displays()
                .into_iter()
                .map(|d| DisplayDto {
                    id: d.id,
                    name: d.name,
                    is_main: d.is_main,
                })
                .collect();
            Response::DisplayList { displays }
        }
    }
}
