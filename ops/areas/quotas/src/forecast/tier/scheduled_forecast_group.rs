use super::*;

use acme_kit::text;

/// Where a scheduled forecast group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledForecastGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledForecastGroupStatus {
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

impl std::fmt::Display for ScheduledForecastGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled forecast group as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastGroup {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub batch_size: i64,
    pub revision: i64,
    pub status: ScheduledForecastGroupStatus,
}

impl ScheduledForecastGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledForecastGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledForecastGroupStatus) -> bool {
        match self.status {
            ScheduledForecastGroupStatus::Open => {
                return next == ScheduledForecastGroupStatus::InReview
                    || next == ScheduledForecastGroupStatus::Resolved;
            }
            ScheduledForecastGroupStatus::InReview => {
                return next == ScheduledForecastGroupStatus::Reopened
                    || next == ScheduledForecastGroupStatus::Archived;
            }
            ScheduledForecastGroupStatus::Resolved => {
                match next {
                    ScheduledForecastGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledForecastGroupStatus::Reopened => {
                match next {
                    ScheduledForecastGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledForecastGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled forecast group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledForecastGroup {
        ScheduledForecastGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            batch_size: self.batch_size,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled forecast group in status open.
pub fn new_scheduled_forecast_group(code: &str, label: &str) -> ScheduledForecastGroup {
    ScheduledForecastGroup {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        batch_size: 0,
        revision: 0,
        status: ScheduledForecastGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_forecast_group("code-3", "label-354");
        assert_eq!(item.status, ScheduledForecastGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_forecast_group("code-3", "label-354");
        if item.move_to(ScheduledForecastGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledForecastGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_forecast_group("code-3", "label-354");
        item.status = ScheduledForecastGroupStatus::Archived;
        assert!(
            item.move_to(ScheduledForecastGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
