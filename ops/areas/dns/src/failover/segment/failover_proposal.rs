use super::*;

use acme_kit::text;

/// Where a failover proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FailoverProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for FailoverProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposal {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub active: bool,
    pub cost: f64,
    pub cipher: String,
    pub fingerprint: String,
    pub capacity: i64,
    pub status: FailoverProposalStatus,
}

impl FailoverProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverProposalStatus) -> bool {
        match self.status {
            FailoverProposalStatus::Idle => {
                match next {
                    FailoverProposalStatus::Ready | FailoverProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverProposalStatus::Warming => {
                match next {
                    FailoverProposalStatus::Draining | FailoverProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverProposalStatus::Ready => {
                match next {
                    FailoverProposalStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverProposalStatus::Draining => {
                return next == FailoverProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverProposal {
        FailoverProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            active: self.active,
            cost: self.cost,
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a failover proposal in status idle.
pub fn new_failover_proposal(code: &str, title: &str) -> FailoverProposal {
    FailoverProposal {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        active: false,
        cost: 0.0,
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        capacity: 0,
        status: FailoverProposalStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_failover_proposal("code-919", "title-58");
        assert_eq!(item.status, FailoverProposalStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_failover_proposal("code-919", "title-58");
        if item.move_to(FailoverProposalStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverProposalStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_failover_proposal("code-919", "title-58");
        item.status = FailoverProposalStatus::Stopped;
        assert!(
            item.move_to(FailoverProposalStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
