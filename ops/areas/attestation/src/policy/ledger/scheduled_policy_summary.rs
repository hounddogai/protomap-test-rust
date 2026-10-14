use super::*;

use acme_kit::text;

/// Where a scheduled policy summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPolicySummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledPolicySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledPolicySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled policy summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPolicySummary {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub threshold: f64,
    pub category: String,
    pub floor_value: f64,
    pub locale: String,
    pub maximum: f64,
    pub limit: i64,
    pub namespace: String,
    pub status: ScheduledPolicySummaryStatus,
}

impl ScheduledPolicySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPolicySummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPolicySummaryStatus) -> bool {
        match self.status {
            ScheduledPolicySummaryStatus::Requested => {
                return next == ScheduledPolicySummaryStatus::Revoked;
            }
            ScheduledPolicySummaryStatus::Granted => {
                return next == ScheduledPolicySummaryStatus::Revoked || next == ScheduledPolicySummaryStatus::Expired;
            }
            ScheduledPolicySummaryStatus::Revoked => {
                match next {
                    ScheduledPolicySummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledPolicySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled policy summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.score - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledPolicySummary {
        ScheduledPolicySummary {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            threshold: self.threshold,
            category: self.category.clone(),
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            maximum: self.maximum,
            limit: self.limit,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled policy summary in status requested.
pub fn new_scheduled_policy_summary(code: &str, label: &str) -> ScheduledPolicySummary {
    ScheduledPolicySummary {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        threshold: 0.0,
        category: "".to_string(),
        floor_value: 0.0,
        locale: "".to_string(),
        maximum: 0.0,
        limit: 0,
        namespace: "".to_string(),
        status: ScheduledPolicySummaryStatus::Requested,
    }
}
