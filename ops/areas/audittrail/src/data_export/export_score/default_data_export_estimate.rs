use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default data export estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDataExportEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultDataExportEstimateStatus {
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

impl std::fmt::Display for DefaultDataExportEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default data export estimate as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDataExportEstimate {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub cluster: String,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub category: String,
    pub status: DefaultDataExportEstimateStatus,
}

impl DefaultDataExportEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDataExportEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDataExportEstimateStatus) -> bool {
        match self.status {
            DefaultDataExportEstimateStatus::Open => {
                return next == DefaultDataExportEstimateStatus::Resolved;
            }
            DefaultDataExportEstimateStatus::InReview => {
                match next {
                    DefaultDataExportEstimateStatus::Resolved | DefaultDataExportEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDataExportEstimateStatus::Resolved => {
                match next {
                    DefaultDataExportEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDataExportEstimateStatus::Reopened => {
                return next == DefaultDataExportEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultDataExportEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default data export estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDataExportEstimate {
        DefaultDataExportEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default data export estimate in status open.
pub fn new_default_data_export_estimate(code: &str, label: &str) -> DefaultDataExportEstimate {
    DefaultDataExportEstimate {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        attributes: BTreeMap::new(),
        grace_days: 0,
        cluster: "".to_string(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        category: "".to_string(),
        status: DefaultDataExportEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_data_export_estimate("code-218", "label-311");
        assert_eq!(item.status, DefaultDataExportEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_data_export_estimate("code-218", "label-311");
        if item.move_to(DefaultDataExportEstimateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDataExportEstimateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_data_export_estimate("code-218", "label-311");
        item.status = DefaultDataExportEstimateStatus::Archived;
        assert!(
            item.move_to(DefaultDataExportEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
