use crate::state::BarItem;

#[derive(Debug, uniffi::Enum)]
pub enum StateChangeEvent {
    ItemAdded { item: BarItem },
    ItemRemoved { name: String },
    ItemUpdated { item: BarItem },
    FullRefresh { items: Vec<BarItem> },
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
