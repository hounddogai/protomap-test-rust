use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a data export policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportPolicyStatus {
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

impl std::fmt::Display for DataExportPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPolicy {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub key_id: String,
    pub contact_email: String,
    pub currency: String,
    pub attributes: BTreeMap<String, String>,
    pub rate: f64,
    pub minimum: f64,
    pub attempts: i64,
    pub locale: String,
    pub status: DataExportPolicyStatus,
}

impl DataExportPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportPolicyStatus) -> bool {
        match self.status {
            DataExportPolicyStatus::Planned => {
                return next == DataExportPolicyStatus::Completed;
            }
            DataExportPolicyStatus::Scheduled => {
                match next {
                    DataExportPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportPolicyStatus::InProgress => {
                match next {
                    DataExportPolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportPolicyStatus::Completed => {
                match next {
                    DataExportPolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DataExportPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.minimum - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportPolicy {
        DataExportPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            attributes: self.attributes.clone(),
            rate: self.rate,
            minimum: self.minimum,
            attempts: self.attempts,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a data export policy in status planned.
pub fn new_data_export_policy(code: &str, label: &str) -> DataExportPolicy {
    DataExportPolicy {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        contact_email: "".to_string(),
        currency: "".to_string(),
        attributes: BTreeMap::new(),
        rate: 0.0,
        minimum: 0.0,
        attempts: 0,
        locale: "".to_string(),
        status: DataExportPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_policy("code-763", "label-824");
        assert_eq!(item.status, DataExportPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_data_export_policy("code-763", "label-824");
        if item.move_to(DataExportPolicyStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportPolicyStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_policy("code-763", "label-824");
        item.status = DataExportPolicyStatus::Cancelled;
        assert!(
            item.move_to(DataExportPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
