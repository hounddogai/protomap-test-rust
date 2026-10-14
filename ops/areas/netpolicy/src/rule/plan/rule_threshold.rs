use super::*;

use acme_kit::text;

/// Where a rule threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RuleThresholdStatus {
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

impl std::fmt::Display for RuleThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleThreshold {
    pub code: String,
    pub label: String,
    pub source: String,
    pub cost: f64,
    pub retries: i64,
    pub status: RuleThresholdStatus,
}

impl RuleThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleThresholdStatus) -> bool {
        match self.status {
            RuleThresholdStatus::New => {
                match next {
                    RuleThresholdStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleThresholdStatus::Queued => {
                return next == RuleThresholdStatus::Failed;
            }
            RuleThresholdStatus::Running => {
                return next == RuleThresholdStatus::Done;
            }
            RuleThresholdStatus::Done => {
                return next == RuleThresholdStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule threshold", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.cost + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleThreshold {
        RuleThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            cost: self.cost,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rule threshold in status new.
pub fn new_rule_threshold(code: &str, label: &str) -> RuleThreshold {
    RuleThreshold {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        cost: 0.0,
        retries: 0,
        status: RuleThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rule_threshold("code-286", "label-609");
        assert_eq!(item.status, RuleThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rule_threshold("code-286", "label-609");
        if item.move_to(RuleThresholdStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleThresholdStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rule_threshold("code-286", "label-609");
        item.status = RuleThresholdStatus::Failed;
        assert!(item.move_to(RuleThresholdStatus::New).is_err(), "expected move to fail");
    }
}
