use super::*;

use acme_kit::text;

/// Where a alert threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AlertThresholdStatus {
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

impl std::fmt::Display for AlertThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertThreshold {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub discount: f64,
    pub segment: String,
    pub tenant_id: String,
    pub summary: String,
    pub margin: f64,
    pub capacity: i64,
    pub approved: bool,
    pub active: bool,
    pub status: AlertThresholdStatus,
}

impl AlertThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertThresholdStatus) -> bool {
        match self.status {
            AlertThresholdStatus::Open => {
                return next == AlertThresholdStatus::InReview || next == AlertThresholdStatus::Reopened;
            }
            AlertThresholdStatus::InReview => {
                match next {
                    AlertThresholdStatus::Reopened | AlertThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertThresholdStatus::Resolved => {
                match next {
                    AlertThresholdStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertThresholdStatus::Reopened => {
                return next == AlertThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.margin * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertThreshold {
        AlertThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            segment: self.segment.clone(),
            tenant_id: self.tenant_id.clone(),
            summary: self.summary.clone(),
            margin: self.margin,
            capacity: self.capacity,
            approved: self.approved,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a alert threshold in status open.
pub fn new_alert_threshold(code: &str, title: &str) -> AlertThreshold {
    AlertThreshold {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        segment: "".to_string(),
        tenant_id: "".to_string(),
        summary: "".to_string(),
        margin: 0.0,
        capacity: 0,
        approved: false,
        active: false,
        status: AlertThresholdStatus::Open,
    }
}
