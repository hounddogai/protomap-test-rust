use super::*;

use acme_kit::text;

/// Where a egress metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EgressMetricStatus {
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

impl std::fmt::Display for EgressMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetric {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub title: String,
    pub cost: f64,
    pub cipher: String,
    pub locale: String,
    pub quantity: i64,
    pub status: EgressMetricStatus,
}

impl EgressMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressMetricStatus) -> bool {
        match self.status {
            EgressMetricStatus::Open => {
                return next == EgressMetricStatus::InReview;
            }
            EgressMetricStatus::InReview => {
                match next {
                    EgressMetricStatus::Reopened | EgressMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressMetricStatus::Resolved => {
                match next {
                    EgressMetricStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressMetricStatus::Reopened => {
                match next {
                    EgressMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: EgressMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress metric", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressMetric {
        EgressMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            title: self.title.clone(),
            cost: self.cost,
            cipher: self.cipher.clone(),
            locale: self.locale.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a egress metric in status open.
pub fn new_egress_metric(code: &str, label: &str) -> EgressMetric {
    EgressMetric {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        title: "".to_string(),
        cost: 0.0,
        cipher: "".to_string(),
        locale: "".to_string(),
        quantity: 0,
        status: EgressMetricStatus::Open,
    }
}
