use super::*;

use acme_kit::text;

/// Where a rule exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RuleExceptionStatus {
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

impl std::fmt::Display for RuleExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule exception as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleException {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub contact_email: String,
    pub starts_at: i64,
    pub ratio: f64,
    pub score: f64,
    pub margin: f64,
    pub rotation_days: i64,
    pub status: RuleExceptionStatus,
}

impl RuleException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleExceptionStatus) -> bool {
        match self.status {
            RuleExceptionStatus::Planned => {
                match next {
                    RuleExceptionStatus::Scheduled | RuleExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleExceptionStatus::Scheduled => {
                return next == RuleExceptionStatus::InProgress;
            }
            RuleExceptionStatus::InProgress => {
                return next == RuleExceptionStatus::Completed;
            }
            RuleExceptionStatus::Completed => {
                return next == RuleExceptionStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule exception", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.starts_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleException {
        RuleException {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            ratio: self.ratio,
            score: self.score,
            margin: self.margin,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule exception in status planned.
pub fn new_rule_exception(code: &str, title: &str) -> RuleException {
    RuleException {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        contact_email: "".to_string(),
        starts_at: 0,
        ratio: 0.0,
        score: 0.0,
        margin: 0.0,
        rotation_days: 0,
        status: RuleExceptionStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rule_exception("code-242", "title-13");
        assert_eq!(item.status, RuleExceptionStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rule_exception("code-242", "title-13");
        if item.move_to(RuleExceptionStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleExceptionStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rule_exception("code-242", "title-13");
        item.status = RuleExceptionStatus::Cancelled;
        assert!(
            item.move_to(RuleExceptionStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
