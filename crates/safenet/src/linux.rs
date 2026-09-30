#[cfg(target_os = "linux")]
use crate::{
    backend::{NetworkBackend, NetworkBackendError},
    observed::ObservedNetworkState,
};

#[cfg(target_os = "linux")]
pub struct LinuxNetworkBackend;

#[cfg(target_os = "linux")]
impl NetworkBackend for LinuxNetworkBackend {
    fn create_isolated_environment(&self, _name: &str) -> Result<(), NetworkBackendError> {
        todo!("create Linux network namespace")
    }

    fn observe(&self, _name: &str) -> Result<ObservedNetworkState, NetworkBackendError> {
        todo!("observe Linux network namespace")
    }

    fn destroy(&self, _name: &str) -> Result<(), NetworkBackendError> {
        todo!("destroy Linux network namespace")
    }
}
