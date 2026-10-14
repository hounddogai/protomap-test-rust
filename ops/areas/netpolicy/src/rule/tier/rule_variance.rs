use super::*;

use acme_kit::text;

/// Where a rule variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RuleVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RuleVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule variance as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleVariance {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub phone: String,
    pub source: String,
    pub label: String,
    pub currency: String,
    pub attempts: i64,
    pub status: RuleVarianceStatus,
}

impl RuleVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleVarianceStatus) -> bool {
        match self.status {
            RuleVarianceStatus::New => {
                match next {
                    RuleVarianceStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleVarianceStatus::Queued => {
                return next == RuleVarianceStatus::Running || next == RuleVarianceStatus::Done;
            }
            RuleVarianceStatus::Running => {
                return next == RuleVarianceStatus::Done || next == RuleVarianceStatus::Failed;
            }
            RuleVarianceStatus::Done => {
                return next == RuleVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> RuleVariance {
        RuleVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            phone: self.phone.clone(),
            source: self.source.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rule variance in status new.
pub fn new_rule_variance(code: &str, name: &str) -> RuleVariance {
    RuleVariance {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        phone: "".to_string(),
        source: "".to_string(),
        label: "".to_string(),
        currency: "".to_string(),
        attempts: 0,
        status: RuleVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rule_variance("code-867", "name-351");
        assert_eq!(item.status, RuleVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rule_variance("code-867", "name-351");
        if item.move_to(RuleVarianceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleVarianceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rule_variance("code-867", "name-351");
        item.status = RuleVarianceStatus::Failed;
        assert!(item.move_to(RuleVarianceStatus::New).is_err(), "expected move to fail");
    }
}
