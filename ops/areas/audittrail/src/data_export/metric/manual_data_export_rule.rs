use super::*;

use acme_kit::text;

/// Where a manual data export rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualDataExportRuleStatus {
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

impl std::fmt::Display for ManualDataExportRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual data export rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportRule {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub capacity: i64,
    pub limit: i64,
    pub attempts: i64,
    pub active: bool,
    pub key_id: String,
    pub fingerprint: String,
    pub quota_units: i64,
    pub timezone: String,
    pub status: ManualDataExportRuleStatus,
}

impl ManualDataExportRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportRuleStatus) -> bool {
        match self.status {
            ManualDataExportRuleStatus::Open => {
                match next {
                    ManualDataExportRuleStatus::InReview | ManualDataExportRuleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportRuleStatus::InReview => {
                match next {
                    ManualDataExportRuleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportRuleStatus::Resolved => {
                return next == ManualDataExportRuleStatus::Archived;
            }
            ManualDataExportRuleStatus::Reopened => {
                return next == ManualDataExportRuleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDataExportRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.capacity as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDataExportRule {
        ManualDataExportRule {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            capacity: self.capacity,
            limit: self.limit,
            attempts: self.attempts,
            active: self.active,
            key_id: self.key_id.clone(),
            fingerprint: self.fingerprint.clone(),
            quota_units: self.quota_units,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual data export rule in status open.
pub fn new_manual_data_export_rule(code: &str, label: &str) -> ManualDataExportRule {
    ManualDataExportRule {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        capacity: 0,
        limit: 0,
        attempts: 0,
        active: false,
        key_id: "".to_string(),
        fingerprint: "".to_string(),
        quota_units: 0,
        timezone: "".to_string(),
        status: ManualDataExportRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_data_export_rule("code-842", "label-657");
        assert_eq!(item.status, ManualDataExportRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_data_export_rule("code-842", "label-657");
        if item.move_to(ManualDataExportRuleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDataExportRuleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_data_export_rule("code-842", "label-657");
        item.status = ManualDataExportRuleStatus::Archived;
        assert!(
            item.move_to(ManualDataExportRuleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
