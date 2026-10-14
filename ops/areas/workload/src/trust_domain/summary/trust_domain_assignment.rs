use super::*;

use acme_kit::text;

/// Where a trust domain assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TrustDomainAssignmentStatus {
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

impl std::fmt::Display for TrustDomainAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAssignment {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub batch_size: i64,
    pub cost: f64,
    pub status: TrustDomainAssignmentStatus,
}

impl TrustDomainAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainAssignmentStatus) -> bool {
        match self.status {
            TrustDomainAssignmentStatus::Idle => {
                match next {
                    TrustDomainAssignmentStatus::Warming | TrustDomainAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAssignmentStatus::Warming => {
                match next {
                    TrustDomainAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAssignmentStatus::Ready => {
                return next == TrustDomainAssignmentStatus::Draining || next == TrustDomainAssignmentStatus::Stopped;
            }
            TrustDomainAssignmentStatus::Draining => {
                match next {
                    TrustDomainAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain assignment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.cost - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainAssignment {
        TrustDomainAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            batch_size: self.batch_size,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a trust domain assignment in status idle.
pub fn new_trust_domain_assignment(code: &str, name: &str) -> TrustDomainAssignment {
    TrustDomainAssignment {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        batch_size: 0,
        cost: 0.0,
        status: TrustDomainAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_trust_domain_assignment("code-916", "name-52");
        assert_eq!(item.status, TrustDomainAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_trust_domain_assignment("code-916", "name-52");
        if item.move_to(TrustDomainAssignmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainAssignmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_trust_domain_assignment("code-916", "name-52");
        item.status = TrustDomainAssignmentStatus::Stopped;
        assert!(
            item.move_to(TrustDomainAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
