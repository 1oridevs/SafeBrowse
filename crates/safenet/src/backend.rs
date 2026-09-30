use crate::observed::ObservedNetworkState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkBackendError {
    UnsupportedPlatform,
    CreateFailed(String),
    ObserveFailed(String),
    DestroyFailed(String),
}

pub trait NetworkBackend {
    fn create_isolated_environment(&self, name: &str) -> Result<(), NetworkBackendError>;

    fn observe(&self, name: &str) -> Result<ObservedNetworkState, NetworkBackendError>;

    fn destroy(&self, name: &str) -> Result<(), NetworkBackendError>;
}
