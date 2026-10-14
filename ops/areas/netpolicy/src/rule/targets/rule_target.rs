use super::*;

use acme_kit::text;

/// Where a rule target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RuleTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RuleTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleTarget {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub attempts: i64,
    pub namespace: String,
    pub version: i64,
    pub currency: String,
    pub expires_at: i64,
    pub maximum: f64,
    pub status: RuleTargetStatus,
}

impl RuleTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleTargetStatus) -> bool {
        match self.status {
            RuleTargetStatus::Planned => {
                match next {
                    RuleTargetStatus::InProgress | RuleTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleTargetStatus::Scheduled => {
                match next {
                    RuleTargetStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleTargetStatus::InProgress => {
                match next {
                    RuleTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleTargetStatus::Completed => {
                return next == RuleTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule target", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleTarget {
        RuleTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            version: self.version,
            currency: self.currency.clone(),
            expires_at: self.expires_at,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rule target in status planned.
pub fn new_rule_target(code: &str, title: &str) -> RuleTarget {
    RuleTarget {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        attempts: 0,
        namespace: "".to_string(),
        version: 0,
        currency: "".to_string(),
        expires_at: 0,
        maximum: 0.0,
        status: RuleTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rule_target("code-906", "title-364");
        assert_eq!(item.status, RuleTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rule_target("code-906", "title-364");
        if item.move_to(RuleTargetStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleTargetStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rule_target("code-906", "title-364");
        item.status = RuleTargetStatus::Cancelled;
        assert!(
            item.move_to(RuleTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
