use super::*;

use acme_kit::text;

/// Where a weighted rule record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRuleRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedRuleRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for WeightedRuleRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted rule record as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecord {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub revision: i64,
    pub ratio: f64,
    pub principal: String,
    pub status: WeightedRuleRecordStatus,
}

impl WeightedRuleRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRuleRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRuleRecordStatus) -> bool {
        match self.status {
            WeightedRuleRecordStatus::Open => {
                match next {
                    WeightedRuleRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleRecordStatus::InReview => {
                return next == WeightedRuleRecordStatus::Resolved || next == WeightedRuleRecordStatus::Archived;
            }
            WeightedRuleRecordStatus::Resolved => {
                match next {
                    WeightedRuleRecordStatus::Reopened | WeightedRuleRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleRecordStatus::Reopened => {
                return next == WeightedRuleRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRuleRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rule record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.revision as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRuleRecord {
        WeightedRuleRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            revision: self.revision,
            ratio: self.ratio,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted rule record in status open.
pub fn new_weighted_rule_record(code: &str, label: &str) -> WeightedRuleRecord {
    WeightedRuleRecord {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        revision: 0,
        ratio: 0.0,
        principal: "".to_string(),
        status: WeightedRuleRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_rule_record("code-605", "label-341");
        assert_eq!(item.status, WeightedRuleRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_weighted_rule_record("code-605", "label-341");
        if item.move_to(WeightedRuleRecordStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRuleRecordStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_rule_record("code-605", "label-341");
        item.status = WeightedRuleRecordStatus::Archived;
        assert!(
            item.move_to(WeightedRuleRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
