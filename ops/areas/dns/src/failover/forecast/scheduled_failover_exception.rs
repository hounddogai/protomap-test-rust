use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled failover exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFailoverExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledFailoverExceptionStatus {
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

impl std::fmt::Display for ScheduledFailoverExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled failover exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverException {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub contact_email: String,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub threshold: f64,
    pub amount: f64,
    pub status: ScheduledFailoverExceptionStatus,
}

impl ScheduledFailoverException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFailoverExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFailoverExceptionStatus) -> bool {
        match self.status {
            ScheduledFailoverExceptionStatus::Idle => {
                match next {
                    ScheduledFailoverExceptionStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFailoverExceptionStatus::Warming => {
                return next == ScheduledFailoverExceptionStatus::Draining
                    || next == ScheduledFailoverExceptionStatus::Stopped;
            }
            ScheduledFailoverExceptionStatus::Ready => {
                return next == ScheduledFailoverExceptionStatus::Draining
                    || next == ScheduledFailoverExceptionStatus::Stopped;
            }
            ScheduledFailoverExceptionStatus::Draining => {
                return next == ScheduledFailoverExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFailoverExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled failover exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn projected_value(&self) -> f64 {
        (self.amount * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFailoverException {
        ScheduledFailoverException {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            contact_email: self.contact_email.clone(),
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            threshold: self.threshold,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled failover exception in status idle.
pub fn new_scheduled_failover_exception(code: &str, label: &str) -> ScheduledFailoverException {
    ScheduledFailoverException {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        contact_email: "".to_string(),
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        fingerprint: "".to_string(),
        threshold: 0.0,
        amount: 0.0,
        status: ScheduledFailoverExceptionStatus::Idle,
    }
}
