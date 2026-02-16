use crate::state::BarItem;

#[derive(Debug, Clone, uniffi::Record)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub is_main: bool,
}

#[derive(Debug, uniffi::Enum)]
pub enum StateChangeEvent {
    ItemAdded { display: u32, item: BarItem },
    ItemRemoved { display: u32, name: String },
    ItemUpdated { display: u32, item: BarItem },
    ItemMoved { old_display: u32, new_display: u32, item: BarItem },
    FullRefresh { display: u32, items: Vec<BarItem> },
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RanmaError {
    #[error("{message}")]
    General { message: String },
}

impl From<uniffi::UnexpectedUniFFICallbackError> for RanmaError {
    fn from(e: uniffi::UnexpectedUniFFICallbackError) -> Self {
        RanmaError::General {
            message: e.to_string(),
        }
    }
}

#[uniffi::export(with_foreign)]
pub trait StateChangeHandler: Send + Sync {
    fn on_state_change(&self, event: StateChangeEvent) -> Result<(), RanmaError>;
}
