use super::*;

use acme_kit::text;

/// Where a failover summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSummary {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub issuer: String,
    pub description: String,
    pub discount: f64,
    pub window_days: i64,
    pub ratio: f64,
    pub batch_size: i64,
    pub visible: bool,
    pub status: FailoverSummaryStatus,
}

impl FailoverSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverSummaryStatus) -> bool {
        match self.status {
            FailoverSummaryStatus::Proposed => {
                return next == FailoverSummaryStatus::Approved || next == FailoverSummaryStatus::Withdrawn;
            }
            FailoverSummaryStatus::Approved => {
                return next == FailoverSummaryStatus::Rejected || next == FailoverSummaryStatus::Withdrawn;
            }
            FailoverSummaryStatus::Rejected => {
                match next {
                    FailoverSummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FailoverSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "failover summary", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.discount - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverSummary {
        FailoverSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            issuer: self.issuer.clone(),
            description: self.description.clone(),
            discount: self.discount,
            window_days: self.window_days,
            ratio: self.ratio,
            batch_size: self.batch_size,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover summary in status proposed.
pub fn new_failover_summary(code: &str, label: &str) -> FailoverSummary {
    FailoverSummary {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        issuer: "".to_string(),
        description: "".to_string(),
        discount: 0.0,
        window_days: 0,
        ratio: 0.0,
        batch_size: 0,
        visible: false,
        status: FailoverSummaryStatus::Proposed,
    }
}
