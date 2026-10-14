use super::*;

use acme_kit::text;

/// Where a rule request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RuleRequestStatus {
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

impl std::fmt::Display for RuleRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule request as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRequest {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub unit: String,
    pub fingerprint: String,
    pub score: f64,
    pub status: RuleRequestStatus,
}

impl RuleRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleRequestStatus) -> bool {
        match self.status {
            RuleRequestStatus::Idle => {
                match next {
                    RuleRequestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleRequestStatus::Warming => {
                return next == RuleRequestStatus::Stopped;
            }
            RuleRequestStatus::Ready => {
                match next {
                    RuleRequestStatus::Draining | RuleRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleRequestStatus::Draining => {
                match next {
                    RuleRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RuleRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule request", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RuleRequest {
        RuleRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            fingerprint: self.fingerprint.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rule request in status idle.
pub fn new_rule_request(code: &str, label: &str) -> RuleRequest {
    RuleRequest {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        unit: "".to_string(),
        fingerprint: "".to_string(),
        score: 0.0,
        status: RuleRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rule_request("code-734", "label-796");
        assert_eq!(item.status, RuleRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_rule_request("code-734", "label-796");
        if item.move_to(RuleRequestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleRequestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rule_request("code-734", "label-796");
        item.status = RuleRequestStatus::Stopped;
        assert!(item.move_to(RuleRequestStatus::Idle).is_err(), "expected move to fail");
    }
}
