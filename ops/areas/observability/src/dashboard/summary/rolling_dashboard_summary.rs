use super::*;

use acme_kit::text;

/// Where a rolling dashboard summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDashboardSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingDashboardSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RollingDashboardSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling dashboard summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummary {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub archived: bool,
    pub cost: f64,
    pub key_id: String,
    pub timezone: String,
    pub owner_id: String,
    pub contact_email: String,
    pub status: RollingDashboardSummaryStatus,
}

impl RollingDashboardSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDashboardSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDashboardSummaryStatus) -> bool {
        match self.status {
            RollingDashboardSummaryStatus::Idle => {
                return next == RollingDashboardSummaryStatus::Warming
                    || next == RollingDashboardSummaryStatus::Draining;
            }
            RollingDashboardSummaryStatus::Warming => {
                return next == RollingDashboardSummaryStatus::Ready || next == RollingDashboardSummaryStatus::Draining;
            }
            RollingDashboardSummaryStatus::Ready => {
                return next == RollingDashboardSummaryStatus::Draining
                    || next == RollingDashboardSummaryStatus::Stopped;
            }
            RollingDashboardSummaryStatus::Draining => {
                match next {
                    RollingDashboardSummaryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingDashboardSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling dashboard summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.cost * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDashboardSummary {
        RollingDashboardSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            archived: self.archived,
            cost: self.cost,
            key_id: self.key_id.clone(),
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling dashboard summary in status idle.
pub fn new_rolling_dashboard_summary(code: &str, label: &str) -> RollingDashboardSummary {
    RollingDashboardSummary {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        archived: false,
        cost: 0.0,
        key_id: "".to_string(),
        timezone: "".to_string(),
        owner_id: "".to_string(),
        contact_email: "".to_string(),
        status: RollingDashboardSummaryStatus::Idle,
    }
}
