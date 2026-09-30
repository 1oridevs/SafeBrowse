use safebrowse_common::{NetworkPolicy, SessionState};

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

    pub fn mark_verified(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Ready)
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

    fn session() -> SafeNetSession {
        SafeNetSession::new(NetworkPolicy::safe_net_1())
    }

    #[test]
    fn verified_session_can_reach_active() {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();
        session.mark_verified().unwrap();
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
    fn verification_failure_can_block_session() {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();
        session.block().unwrap();

        assert_eq!(session.state(), SessionState::Blocked);
    }

    #[test]
    fn blocked_session_cannot_be_reactivated() {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();
        session.block().unwrap();

        assert!(session.activate().is_err());
        assert_eq!(session.state(), SessionState::Blocked);
    }

    #[test]
    fn active_session_cannot_be_destroyed_directly() {
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();
        session.mark_verified().unwrap();
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
        let mut session = session();

        session.mark_isolated().unwrap();
        session.begin_verification().unwrap();
        session.mark_verified().unwrap();
        session.activate().unwrap();
        session.block().unwrap();
        session.destroy().unwrap();

        assert_eq!(session.state(), SessionState::Destroyed);
    }
}
