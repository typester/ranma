uniffi::setup_scaffolding!();

pub mod bridge;
pub mod ipc;
pub mod state;

use std::path::Path;
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;

use bridge::{DisplayInfo, StateChangeEvent, StateChangeHandler};
use state::{BarNode, BarState};

static STATE: OnceLock<Arc<Mutex<BarState>>> = OnceLock::new();
static HANDLER: OnceLock<Arc<dyn StateChangeHandler>> = OnceLock::new();
static DISPLAYS: OnceLock<Arc<Mutex<Vec<DisplayInfo>>>> = OnceLock::new();

pub(crate) fn get_state() -> &'static Arc<Mutex<BarState>> {
    STATE.get_or_init(|| Arc::new(Mutex::new(BarState::default())))
}

fn get_displays_store() -> &'static Arc<Mutex<Vec<DisplayInfo>>> {
    DISPLAYS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

pub fn notify(event: StateChangeEvent) {
    if let Some(handler) = HANDLER.get() {
        let _ = handler.on_state_change(event);
    }
}

#[uniffi::export]
pub fn register_handler(handler: Arc<dyn StateChangeHandler>) {
    let _ = HANDLER.set(handler);
}

#[uniffi::export]
pub fn start_server(socket_path: String) {
    let path = socket_path.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = ipc::server::run(Path::new(&path)).await {
                eprintln!("server error: {e}");
            }
        });
    });
}

#[uniffi::export]
pub fn set_displays(displays: Vec<DisplayInfo>) {
    let old_displays = {
        let mut store = get_displays_store().lock();
        let old = store.clone();
        *store = displays.clone();
        old
    };

    let new_ids: std::collections::HashSet<u32> = displays.iter().map(|d| d.id).collect();
    let removed: Vec<u32> = old_displays
        .iter()
        .map(|d| d.id)
        .filter(|id| !new_ids.contains(id))
        .collect();

    if removed.is_empty() {
        return;
    }

    let new_main = main_display_id();
    if new_main == 0 {
        return;
    }

    let mut events = Vec::new();
    {
        let mut state = get_state().lock();
        for &old_display in &removed {
            for node in state.migrate_nodes(old_display, new_main) {
                events.push(StateChangeEvent::NodeMoved {
                    old_display,
                    new_display: new_main,
                    node,
                });
            }
        }
    }

    for event in events {
        notify(event);
    }
}

#[uniffi::export]
pub fn get_displays() -> Vec<DisplayInfo> {
    get_displays_store().lock().clone()
}

#[uniffi::export]
pub fn get_nodes() -> Vec<BarNode> {
    get_state().lock().get_nodes()
}

#[uniffi::export]
pub fn get_nodes_for_display(display: u32) -> Vec<BarNode> {
    get_state().lock().get_nodes_for_display(display)
}

pub(crate) fn main_display_id() -> u32 {
    get_displays_store()
        .lock()
        .iter()
        .find(|d| d.is_main)
        .map(|d| d.id)
        .unwrap_or(0)
}
