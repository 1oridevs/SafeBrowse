pub mod observed;
pub mod verifier;

use safebrowse_common::{NetworkPolicy, SessionState};

use crate::observed::ObservedNetworkState;
use crate::verifier::{VerificationFailure, verify_initial_isolation};

#[derive(Debug)]
pub struct SafeNetSession {
    state: SessionState,
    policy: NetworkPolicy,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionError {
    InvalidTransition {
        from: SessionState,
        to: SessionState,
    },
}

impl SafeNetSession {
    pub fn new(policy: NetworkPolicy) -> Self {
        Self {
            state: SessionState::Created,
            policy,
        }
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn policy(&self) -> &NetworkPolicy {
        &self.policy
    }

    fn transition(&mut self, next: SessionState) -> Result<(), SessionError> {
        let allowed = matches!(
            (self.state, next),
            (SessionState::Created, SessionState::Isolated)
                | (SessionState::Isolated, SessionState::Verifying)
                | (SessionState::Verifying, SessionState::Ready)
                | (SessionState::Verifying, SessionState::Blocked)
                | (SessionState::Ready, SessionState::Active)
                | (SessionState::Ready, SessionState::Blocked)
                | (SessionState::Active, SessionState::Blocked)
                | (SessionState::Blocked, SessionState::Destroyed)
        );

        if !allowed {
            return Err(SessionError::InvalidTransition {
                from: self.state,
                to: next,
            });
        }

        self.state = next;
        Ok(())
    }

    pub fn mark_isolated(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Isolated)
    }

    pub fn begin_verification(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Verifying)
    }

    pub fn verify(
        &mut self,
        observed: &ObservedNetworkState,
    ) -> Result<(), Vec<VerificationFailure>> {
        if self.state != SessionState::Verifying {
            return Err(vec![VerificationFailure::InvalidSessionState {
                actual: self.state,
            }]);
        }

        match verify_initial_isolation(observed) {
            Ok(()) => {
                self.state = SessionState::Ready;
                Ok(())
            }
            Err(failures) => {
                self.state = SessionState::Blocked;
                Err(failures)
            }
        }
    }

    pub fn activate(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Active)
    }

    pub fn block(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Blocked)
    }

    pub fn destroy(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Destroyed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observed::{NetworkInterface, ObservedNetworkState};

    fn session() -> SafeNetSession {
        SafeNetSession::new(NetworkPolicy::safe_net_1())
    }

    fn verified_session() -> SafeNetSession {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();
        session
            .verify(&ObservedNetworkState::isolated_loopback_only())
            .unwrap();

        session
    }

    #[test]
    fn verified_session_can_reach_active() {
        let mut session = verified_session();

        session.activate().unwrap();

        assert_eq!(session.state(), SessionState::Active);
    }

    #[test]
    fn created_session_cannot_activate() {
        let mut session = session();

        assert_eq!(
            session.activate(),
            Err(SessionError::InvalidTransition {
                from: SessionState::Created,
                to: SessionState::Active,
            })
        );
    }

    #[test]
    fn unverified_session_cannot_activate() {
        let mut session = session();

        session.mark_isolated().unwrap();

        assert_eq!(
            session.activate(),
            Err(SessionError::InvalidTransition {
                from: SessionState::Isolated,
                to: SessionState::Active,
            })
        );
    }

    #[test]
    fn unsafe_observed_state_blocks_session() {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();

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
            default_ipv4_route: true,
            default_ipv6_route: false,
            ipv6_enabled: false,
        };

        let failures = session.verify(&observed).unwrap_err();

        assert_eq!(session.state(), SessionState::Blocked);
        assert!(failures.contains(&VerificationFailure::UnexpectedInterface("eth0".into())));
        assert!(failures.contains(&VerificationFailure::DefaultIpv4Route));
    }

    #[test]
    fn verification_cannot_run_before_verifying_state() {
        let mut session = session();

        let result = session.verify(&ObservedNetworkState::isolated_loopback_only());

        assert_eq!(
            result,
            Err(vec![VerificationFailure::InvalidSessionState {
                actual: SessionState::Created,
            }])
        );

        assert_eq!(session.state(), SessionState::Created);
    }

    #[test]
    fn blocked_session_cannot_be_reactivated() {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();

        let mut observed = ObservedNetworkState::isolated_loopback_only();
        observed.default_ipv4_route = true;

        assert!(session.verify(&observed).is_err());
        assert_eq!(session.state(), SessionState::Blocked);

        assert!(session.activate().is_err());
        assert_eq!(session.state(), SessionState::Blocked);
    }

    #[test]
    fn active_session_cannot_be_destroyed_directly() {
        let mut session = verified_session();

        session.activate().unwrap();

        assert_eq!(
            session.destroy(),
            Err(SessionError::InvalidTransition {
                from: SessionState::Active,
                to: SessionState::Destroyed,
            })
        );

        assert_eq!(session.state(), SessionState::Active);
    }

    #[test]
    fn blocked_session_can_be_destroyed() {
        let mut session = verified_session();

        session.activate().unwrap();
        session.block().unwrap();
        session.destroy().unwrap();

        assert_eq!(session.state(), SessionState::Destroyed);
    }
}
