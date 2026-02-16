use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use crate::bridge::StateChangeEvent;
use crate::state::BarItem;
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
            }
        }
    };

    match cmd {
        Command::Add {
            name,
            label,
            icon,
            icon_color,
            background_color,
            position,
            display,
        } => {
            let display = display.unwrap_or_else(main_display_id);
            let item = BarItem {
                name,
                label,
                icon,
                icon_color,
                background_color,
                position: position.unwrap_or(0),
                display,
            };
            let mut state = get_state().lock();
            match state.add_item(item.clone()) {
                Ok(()) => {
                    drop(state);
                    notify(StateChangeEvent::ItemAdded { display, item });
                    Response::Ok
                }
                Err(message) => Response::Error { message },
            }
        }
        Command::Set { name, properties } => {
            let mut state = get_state().lock();
            let old_display = state
                .get_items()
                .iter()
                .find(|i| i.name == name)
                .map(|i| i.display);

            match state.set_properties(&name, &properties) {
                Ok(item) => {
                    let new_display = item.display;
                    drop(state);

                    if let Some(old) = old_display {
                        if old != new_display {
                            notify(StateChangeEvent::ItemMoved {
                                old_display: old,
                                new_display,
                                item,
                            });
                        } else {
                            notify(StateChangeEvent::ItemUpdated {
                                display: new_display,
                                item,
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
            match state.remove_item(&name) {
                Ok(item) => {
                    let display = item.display;
                    drop(state);
                    notify(StateChangeEvent::ItemRemoved {
                        display,
                        name,
                    });
                    Response::Ok
                }
                Err(message) => Response::Error { message },
            }
        }
        Command::Query { name, display } => {
            let state = get_state().lock();
            let items: Vec<_> = match (name, display) {
                (Some(name), _) => state
                    .get_items()
                    .into_iter()
                    .filter(|i| i.name == name)
                    .map(Into::into)
                    .collect(),
                (None, Some(display)) => state
                    .get_items_for_display(display)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                (None, None) => state.get_items().into_iter().map(Into::into).collect(),
            };
            Response::QueryResult { items }
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
