use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled verifier summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledVerifierSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledVerifierSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ScheduledVerifierSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled verifier summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummary {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub attributes: BTreeMap<String, String>,
    pub timezone: String,
    pub tenant_id: String,
    pub limit: i64,
    pub notes: String,
    pub status: ScheduledVerifierSummaryStatus,
}

impl ScheduledVerifierSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledVerifierSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledVerifierSummaryStatus) -> bool {
        match self.status {
            ScheduledVerifierSummaryStatus::Planned => {
                return next == ScheduledVerifierSummaryStatus::Cancelled;
            }
            ScheduledVerifierSummaryStatus::Scheduled => {
                return next == ScheduledVerifierSummaryStatus::InProgress
                    || next == ScheduledVerifierSummaryStatus::Completed;
            }
            ScheduledVerifierSummaryStatus::InProgress => {
                match next {
                    ScheduledVerifierSummaryStatus::Completed | ScheduledVerifierSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledVerifierSummaryStatus::Completed => {
                return next == ScheduledVerifierSummaryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledVerifierSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled verifier summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> ScheduledVerifierSummary {
        ScheduledVerifierSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            attributes: self.attributes.clone(),
            timezone: self.timezone.clone(),
            tenant_id: self.tenant_id.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled verifier summary in status planned.
pub fn new_scheduled_verifier_summary(code: &str, label: &str) -> ScheduledVerifierSummary {
    ScheduledVerifierSummary {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        attributes: BTreeMap::new(),
        timezone: "".to_string(),
        tenant_id: "".to_string(),
        limit: 0,
        notes: "".to_string(),
        status: ScheduledVerifierSummaryStatus::Planned,
    }
}
