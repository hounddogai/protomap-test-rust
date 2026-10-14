use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending alert batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingAlertBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingAlertBatchStatus {
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

impl std::fmt::Display for PendingAlertBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending alert batch as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAlertBatch {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub channel: String,
    pub sequence: i64,
    pub notes: String,
    pub principal: String,
    pub quantity: i64,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub status: PendingAlertBatchStatus,
}

impl PendingAlertBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingAlertBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingAlertBatchStatus) -> bool {
        match self.status {
            PendingAlertBatchStatus::Idle => {
                match next {
                    PendingAlertBatchStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAlertBatchStatus::Warming => {
                match next {
                    PendingAlertBatchStatus::Draining | PendingAlertBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAlertBatchStatus::Ready => {
                return next == PendingAlertBatchStatus::Draining || next == PendingAlertBatchStatus::Stopped;
            }
            PendingAlertBatchStatus::Draining => {
                match next {
                    PendingAlertBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingAlertBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending alert batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn projected_load(&self) -> f64 {
        ((self.sequence as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingAlertBatch {
        PendingAlertBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            channel: self.channel.clone(),
            sequence: self.sequence,
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            quantity: self.quantity,
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending alert batch in status idle.
pub fn new_pending_alert_batch(code: &str, title: &str) -> PendingAlertBatch {
    PendingAlertBatch {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        channel: "".to_string(),
        sequence: 0,
        notes: "".to_string(),
        principal: "".to_string(),
        quantity: 0,
        country: "".to_string(),
        attributes: BTreeMap::new(),
        threshold: 0.0,
        status: PendingAlertBatchStatus::Idle,
    }
}
