use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rule record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule record as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecord {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub notes: String,
    pub window_days: i64,
    pub status: RuleRecordStatus,
}

impl RuleRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleRecordStatus) -> bool {
        match self.status {
            RuleRecordStatus::Proposed => {
                return next == RuleRecordStatus::Approved;
            }
            RuleRecordStatus::Approved => {
                match next {
                    RuleRecordStatus::Rejected | RuleRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleRecordStatus::Rejected => {
                match next {
                    RuleRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RuleRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> RuleRecord {
        RuleRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            notes: self.notes.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule record in status proposed.
pub fn new_rule_record(code: &str, title: &str) -> RuleRecord {
    RuleRecord {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        notes: "".to_string(),
        window_days: 0,
        status: RuleRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rule_record("code-833", "title-861");
        assert_eq!(item.status, RuleRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rule_record("code-833", "title-861");
        if item.move_to(RuleRecordStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleRecordStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rule_record("code-833", "title-861");
        item.status = RuleRecordStatus::Withdrawn;
        assert!(
            item.move_to(RuleRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
