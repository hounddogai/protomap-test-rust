use super::*;

use acme_kit::text;

/// Where a strict rule request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRuleRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictRuleRequestStatus {
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

impl std::fmt::Display for StrictRuleRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict rule request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleRequest {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub currency: String,
    pub archived: bool,
    pub budget: f64,
    pub status: StrictRuleRequestStatus,
}

impl StrictRuleRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRuleRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRuleRequestStatus) -> bool {
        match self.status {
            StrictRuleRequestStatus::Idle => {
                match next {
                    StrictRuleRequestStatus::Ready | StrictRuleRequestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRuleRequestStatus::Warming => {
                match next {
                    StrictRuleRequestStatus::Ready | StrictRuleRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRuleRequestStatus::Ready => {
                return next == StrictRuleRequestStatus::Draining || next == StrictRuleRequestStatus::Stopped;
            }
            StrictRuleRequestStatus::Draining => {
                return next == StrictRuleRequestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictRuleRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rule request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> StrictRuleRequest {
        StrictRuleRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            currency: self.currency.clone(),
            archived: self.archived,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict rule request in status idle.
pub fn new_strict_rule_request(code: &str, name: &str) -> StrictRuleRequest {
    StrictRuleRequest {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        currency: "".to_string(),
        archived: false,
        budget: 0.0,
        status: StrictRuleRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_rule_request("code-158", "name-650");
        assert_eq!(item.status, StrictRuleRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_strict_rule_request("code-158", "name-650");
        if item.move_to(StrictRuleRequestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRuleRequestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_rule_request("code-158", "name-650");
        item.status = StrictRuleRequestStatus::Stopped;
        assert!(
            item.move_to(StrictRuleRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
