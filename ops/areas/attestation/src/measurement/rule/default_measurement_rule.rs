use super::*;

use acme_kit::text;

/// Where a default measurement rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMeasurementRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultMeasurementRuleStatus {
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

impl std::fmt::Display for DefaultMeasurementRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default measurement rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementRule {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub revision: i64,
    pub window_days: i64,
    pub region: String,
    pub notes: String,
    pub pinned: bool,
    pub ceiling: f64,
    pub sequence: i64,
    pub status: DefaultMeasurementRuleStatus,
}

impl DefaultMeasurementRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMeasurementRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMeasurementRuleStatus) -> bool {
        match self.status {
            DefaultMeasurementRuleStatus::Open => {
                match next {
                    DefaultMeasurementRuleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementRuleStatus::InReview => {
                return next == DefaultMeasurementRuleStatus::Reopened;
            }
            DefaultMeasurementRuleStatus::Resolved => {
                return next == DefaultMeasurementRuleStatus::Archived;
            }
            DefaultMeasurementRuleStatus::Reopened => {
                match next {
                    DefaultMeasurementRuleStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultMeasurementRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default measurement rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default measurement rule", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMeasurementRule {
        DefaultMeasurementRule {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            revision: self.revision,
            window_days: self.window_days,
            region: self.region.clone(),
            notes: self.notes.clone(),
            pinned: self.pinned,
            ceiling: self.ceiling,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default measurement rule in status open.
pub fn new_default_measurement_rule(code: &str, title: &str) -> DefaultMeasurementRule {
    DefaultMeasurementRule {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        duration_minutes: 0,
        revision: 0,
        window_days: 0,
        region: "".to_string(),
        notes: "".to_string(),
        pinned: false,
        ceiling: 0.0,
        sequence: 0,
        status: DefaultMeasurementRuleStatus::Open,
    }
}
