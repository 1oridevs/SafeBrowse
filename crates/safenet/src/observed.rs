#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterface {
    pub name: String,
    pub loopback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedNetworkState {
    pub interfaces: Vec<NetworkInterface>,
    pub default_ipv4_route: bool,
    pub default_ipv6_route: bool,
    pub ipv6_enabled: bool,
}

impl ObservedNetworkState {
    pub fn isolated_loopback_only() -> Self {
        Self {
            interfaces: vec![NetworkInterface {
                name: "lo".into(),
                loopback: true,
            }],
            default_ipv4_route: false,
            default_ipv6_route: false,
            ipv6_enabled: false,
        }
    }
}
