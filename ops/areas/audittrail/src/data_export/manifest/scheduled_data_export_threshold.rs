use super::*;

use acme_kit::text;

/// Where a scheduled data export threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDataExportThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledDataExportThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledDataExportThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled data export threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportThreshold {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub revision: i64,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub budget: f64,
    pub status: ScheduledDataExportThresholdStatus,
}

impl ScheduledDataExportThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDataExportThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDataExportThresholdStatus) -> bool {
        match self.status {
            ScheduledDataExportThresholdStatus::Requested => {
                match next {
                    ScheduledDataExportThresholdStatus::Granted | ScheduledDataExportThresholdStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportThresholdStatus::Granted => {
                return next == ScheduledDataExportThresholdStatus::Revoked
                    || next == ScheduledDataExportThresholdStatus::Expired;
            }
            ScheduledDataExportThresholdStatus::Revoked => {
                match next {
                    ScheduledDataExportThresholdStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledDataExportThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled data export threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDataExportThreshold {
        ScheduledDataExportThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            revision: self.revision,
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            pinned: self.pinned,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled data export threshold in status requested.
pub fn new_scheduled_data_export_threshold(code: &str, label: &str) -> ScheduledDataExportThreshold {
    ScheduledDataExportThreshold {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        revision: 0,
        grace_days: 0,
        tags: Vec::new(),
        pinned: false,
        budget: 0.0,
        status: ScheduledDataExportThresholdStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_data_export_threshold("code-10", "label-880");
        assert_eq!(item.status, ScheduledDataExportThresholdStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_scheduled_data_export_threshold("code-10", "label-880");
        if item.move_to(ScheduledDataExportThresholdStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDataExportThresholdStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_data_export_threshold("code-10", "label-880");
        item.status = ScheduledDataExportThresholdStatus::Expired;
        assert!(
            item.move_to(ScheduledDataExportThresholdStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
