use super::*;

use acme_kit::text;

/// Where a pending rule record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRuleRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingRuleRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingRuleRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending rule record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecord {
    pub code: String,
    pub title: String,
    pub category: String,
    pub currency: String,
    pub checksum: String,
    pub duration_minutes: i64,
    pub unit: String,
    pub label: String,
    pub sequence: i64,
    pub retries: i64,
    pub rotation_days: i64,
    pub status: PendingRuleRecordStatus,
}

impl PendingRuleRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRuleRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRuleRecordStatus) -> bool {
        match self.status {
            PendingRuleRecordStatus::Created => {
                match next {
                    PendingRuleRecordStatus::Validated | PendingRuleRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRuleRecordStatus::Validated => {
                return next == PendingRuleRecordStatus::Published || next == PendingRuleRecordStatus::Retired;
            }
            PendingRuleRecordStatus::Published => {
                return next == PendingRuleRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRuleRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending rule record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending rule record", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRuleRecord {
        PendingRuleRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            unit: self.unit.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            retries: self.retries,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending rule record in status created.
pub fn new_pending_rule_record(code: &str, title: &str) -> PendingRuleRecord {
    PendingRuleRecord {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        currency: "".to_string(),
        checksum: "".to_string(),
        duration_minutes: 0,
        unit: "".to_string(),
        label: "".to_string(),
        sequence: 0,
        retries: 0,
        rotation_days: 0,
        status: PendingRuleRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_rule_record("code-241", "title-45");
        assert_eq!(item.status, PendingRuleRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_rule_record("code-241", "title-45");
        if item.move_to(PendingRuleRecordStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRuleRecordStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_rule_record("code-241", "title-45");
        item.status = PendingRuleRecordStatus::Retired;
        assert!(
            item.move_to(PendingRuleRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
