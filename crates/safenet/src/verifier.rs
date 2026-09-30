use crate::observed::ObservedNetworkState;
use safebrowse_common::SessionState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFailure {
    InvalidSessionState { actual: SessionState },
    UnexpectedInterface(String),
    DefaultIpv4Route,
    DefaultIpv6Route,
    Ipv6Enabled,
}

pub fn verify_initial_isolation(
    observed: &ObservedNetworkState,
) -> Result<(), Vec<VerificationFailure>> {
    let mut failures = Vec::new();

    for interface in &observed.interfaces {
        if !interface.loopback {
            failures.push(VerificationFailure::UnexpectedInterface(
                interface.name.clone(),
            ));
        }
    }

    if observed.default_ipv4_route {
        failures.push(VerificationFailure::DefaultIpv4Route);
    }

    if observed.default_ipv6_route {
        failures.push(VerificationFailure::DefaultIpv6Route);
    }

    if observed.ipv6_enabled {
        failures.push(VerificationFailure::Ipv6Enabled);
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::observed::{NetworkInterface, ObservedNetworkState};

    #[test]
    fn loopback_only_state_passes() {
        let observed = ObservedNetworkState::isolated_loopback_only();

        assert_eq!(verify_initial_isolation(&observed), Ok(()));
    }

    #[test]
    fn external_interface_fails() {
        let observed = ObservedNetworkState {
            interfaces: vec![
                NetworkInterface {
                    name: "lo".into(),
                    loopback: true,
                },
                NetworkInterface {
                    name: "eth0".into(),
                    loopback: false,
                },
            ],
            default_ipv4_route: false,
            default_ipv6_route: false,
            ipv6_enabled: false,
        };

        assert_eq!(
            verify_initial_isolation(&observed),
            Err(vec![VerificationFailure::UnexpectedInterface(
                "eth0".into()
            )])
        );
    }

    #[test]
    fn default_ipv4_route_fails() {
        let mut observed = ObservedNetworkState::isolated_loopback_only();
        observed.default_ipv4_route = true;

        assert_eq!(
            verify_initial_isolation(&observed),
            Err(vec![VerificationFailure::DefaultIpv4Route])
        );
    }

    #[test]
    fn ipv6_escape_fails() {
        let mut observed = ObservedNetworkState::isolated_loopback_only();
        observed.ipv6_enabled = true;
        observed.default_ipv6_route = true;

        assert_eq!(
            verify_initial_isolation(&observed),
            Err(vec![
                VerificationFailure::DefaultIpv6Route,
                VerificationFailure::Ipv6Enabled,
            ])
        );
    }

    #[test]
    fn verifier_reports_multiple_failures() {
        let observed = ObservedNetworkState {
            interfaces: vec![NetworkInterface {
                name: "wlan0".into(),
                loopback: false,
            }],
            default_ipv4_route: true,
            default_ipv6_route: true,
            ipv6_enabled: true,
        };

        assert_eq!(
            verify_initial_isolation(&observed),
            Err(vec![
                VerificationFailure::UnexpectedInterface("wlan0".into()),
                VerificationFailure::DefaultIpv4Route,
                VerificationFailure::DefaultIpv6Route,
                VerificationFailure::Ipv6Enabled,
            ])
        );
    }
}
