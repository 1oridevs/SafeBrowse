#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Created,
    Isolated,
    Verifying,
    Ready,
    Active,
    Blocked,
    Destroyed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ipv6Policy {
    Deny,
    AllowThroughTransport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPolicy {
    pub direct_internet: Access,
    pub private_networks: Access,
    pub host_network: Access,
    pub external_dns: Access,
    pub ipv6: Ipv6Policy,
    pub fail_closed: bool,
}

impl NetworkPolicy {
    /// The initial SafeNet policy. No application gets direct network access.
    pub fn safe_net_1() -> Self {
        Self {
            direct_internet: Access::Deny,
            private_networks: Access::Deny,
            host_network: Access::Deny,
            external_dns: Access::Deny,
            ipv6: Ipv6Policy::Deny,
            fail_closed: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_net_1_is_fail_closed() {
        let policy = NetworkPolicy::safe_net_1();

        assert_eq!(policy.direct_internet, Access::Deny);
        assert_eq!(policy.private_networks, Access::Deny);
        assert_eq!(policy.host_network, Access::Deny);
        assert_eq!(policy.external_dns, Access::Deny);
        assert_eq!(policy.ipv6, Ipv6Policy::Deny);
        assert!(policy.fail_closed);
    }
}
