use std::process::{Command, Output};

use crate::{
    backend::{NetworkBackend, NetworkBackendError},
    observed::{NetworkInterface, ObservedNetworkState},
};

pub struct LinuxNetworkBackend;

impl LinuxNetworkBackend {
    fn ip(args: &[&str]) -> std::io::Result<Output> {
        Command::new("ip").args(args).output()
    }

    fn namespace_exists(name: &str) -> Result<bool, NetworkBackendError> {
        let output = Self::ip(&["netns", "list"])
            .map_err(|error| NetworkBackendError::ObserveFailed(error.to_string()))?;

        if !output.status.success() {
            return Err(NetworkBackendError::ObserveFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .any(|namespace| namespace == name))
    }

    fn namespace_output(name: &str, args: &[&str]) -> Result<Output, NetworkBackendError> {
        let mut command_args = vec!["netns", "exec", name, "ip"];
        command_args.extend_from_slice(args);

        Self::ip(&command_args)
            .map_err(|error| NetworkBackendError::ObserveFailed(error.to_string()))
    }
}

impl NetworkBackend for LinuxNetworkBackend {
    fn create_isolated_environment(&self, name: &str) -> Result<(), NetworkBackendError> {
        if Self::namespace_exists(name)? {
            return Err(NetworkBackendError::CreateFailed(format!(
                "network namespace {name} already exists"
            )));
        }

        let create = Self::ip(&["netns", "add", name])
            .map_err(|error| NetworkBackendError::CreateFailed(error.to_string()))?;

        if !create.status.success() {
            return Err(NetworkBackendError::CreateFailed(
                String::from_utf8_lossy(&create.stderr).trim().to_owned(),
            ));
        }

        let loopback = Self::ip(&["netns", "exec", name, "ip", "link", "set", "lo", "up"])
            .map_err(|error| NetworkBackendError::CreateFailed(error.to_string()))?;

        if !loopback.status.success() {
            let _ = Self::ip(&["netns", "delete", name]);

            return Err(NetworkBackendError::CreateFailed(
                String::from_utf8_lossy(&loopback.stderr).trim().to_owned(),
            ));
        }

        Ok(())
    }

    fn observe(&self, name: &str) -> Result<ObservedNetworkState, NetworkBackendError> {
        if !Self::namespace_exists(name)? {
            return Err(NetworkBackendError::ObserveFailed(format!(
                "network namespace {name} does not exist"
            )));
        }

        let links = Self::namespace_output(name, &["-o", "link", "show"])?;

        if !links.status.success() {
            return Err(NetworkBackendError::ObserveFailed(
                String::from_utf8_lossy(&links.stderr).trim().to_owned(),
            ));
        }

        let interfaces = String::from_utf8_lossy(&links.stdout)
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();

                fields.next()?;
                let raw_name = fields.next()?;
                let name = raw_name.trim_end_matches(':').split('@').next()?.to_owned();

                Some(NetworkInterface {
                    loopback: name == "lo",
                    name,
                })
            })
            .collect();

        let ipv4_routes = Self::namespace_output(name, &["-4", "route", "show", "default"])?;

        if !ipv4_routes.status.success() {
            return Err(NetworkBackendError::ObserveFailed(
                String::from_utf8_lossy(&ipv4_routes.stderr)
                    .trim()
                    .to_owned(),
            ));
        }

        let ipv6_routes = Self::namespace_output(name, &["-6", "route", "show", "default"])?;

        if !ipv6_routes.status.success() {
            return Err(NetworkBackendError::ObserveFailed(
                String::from_utf8_lossy(&ipv6_routes.stderr)
                    .trim()
                    .to_owned(),
            ));
        }

        let ipv6_addresses = Self::namespace_output(name, &["-6", "addr", "show"])?;

        if !ipv6_addresses.status.success() {
            return Err(NetworkBackendError::ObserveFailed(
                String::from_utf8_lossy(&ipv6_addresses.stderr)
                    .trim()
                    .to_owned(),
            ));
        }

        Ok(ObservedNetworkState {
            interfaces,
            default_ipv4_route: !ipv4_routes.stdout.is_empty(),
            default_ipv6_route: !ipv6_routes.stdout.is_empty(),
            ipv6_enabled: !ipv6_addresses.stdout.is_empty(),
        })
    }

    fn destroy(&self, name: &str) -> Result<(), NetworkBackendError> {
        if !Self::namespace_exists(name)? {
            return Ok(());
        }

        let output = Self::ip(&["netns", "delete", name])
            .map_err(|error| NetworkBackendError::DestroyFailed(error.to_string()))?;

        if !output.status.success() {
            return Err(NetworkBackendError::DestroyFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }

        Ok(())
    }
}
