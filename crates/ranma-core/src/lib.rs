uniffi::setup_scaffolding!();

pub mod bridge;
pub mod ipc;
pub mod state;

use std::path::Path;
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;

use bridge::{StateChangeEvent, StateChangeHandler};
use state::{BarItem, BarState};

static STATE: OnceLock<Arc<Mutex<BarState>>> = OnceLock::new();
static HANDLER: OnceLock<Arc<dyn StateChangeHandler>> = OnceLock::new();

pub(crate) fn get_state() -> &'static Arc<Mutex<BarState>> {
    STATE.get_or_init(|| Arc::new(Mutex::new(BarState::default())))
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
pub fn get_items() -> Vec<BarItem> {
    get_state().lock().get_items()
}
