use super::*;

use acme_kit::text;

/// Where a fallback data export schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDataExportScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackDataExportScheduleStatus {
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

impl std::fmt::Display for FallbackDataExportScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback data export schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportSchedule {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub segment: String,
    pub currency: String,
    pub timezone: String,
    pub category: String,
    pub status: FallbackDataExportScheduleStatus,
}

impl FallbackDataExportSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDataExportScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDataExportScheduleStatus) -> bool {
        match self.status {
            FallbackDataExportScheduleStatus::Open => {
                match next {
                    FallbackDataExportScheduleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportScheduleStatus::InReview => {
                return next == FallbackDataExportScheduleStatus::Resolved
                    || next == FallbackDataExportScheduleStatus::Reopened;
            }
            FallbackDataExportScheduleStatus::Resolved => {
                return next == FallbackDataExportScheduleStatus::Reopened
                    || next == FallbackDataExportScheduleStatus::Archived;
            }
            FallbackDataExportScheduleStatus::Reopened => {
                match next {
                    FallbackDataExportScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackDataExportScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback data export schedule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> FallbackDataExportSchedule {
        FallbackDataExportSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            currency: self.currency.clone(),
            timezone: self.timezone.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback data export schedule in status open.
pub fn new_fallback_data_export_schedule(code: &str, label: &str) -> FallbackDataExportSchedule {
    FallbackDataExportSchedule {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        segment: "".to_string(),
        currency: "".to_string(),
        timezone: "".to_string(),
        category: "".to_string(),
        status: FallbackDataExportScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_data_export_schedule("code-338", "label-285");
        assert_eq!(item.status, FallbackDataExportScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_fallback_data_export_schedule("code-338", "label-285");
        if item.move_to(FallbackDataExportScheduleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDataExportScheduleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_data_export_schedule("code-338", "label-285");
        item.status = FallbackDataExportScheduleStatus::Archived;
        assert!(
            item.move_to(FallbackDataExportScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
