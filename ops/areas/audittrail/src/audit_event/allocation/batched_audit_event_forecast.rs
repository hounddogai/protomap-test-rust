use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched audit event forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAuditEventForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedAuditEventForecastStatus {
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

impl std::fmt::Display for BatchedAuditEventForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched audit event forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAuditEventForecast {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub capacity: i64,
    pub grace_days: i64,
    pub approved: bool,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub pinned: bool,
    pub namespace: String,
    pub cipher: String,
    pub status: BatchedAuditEventForecastStatus,
}

impl BatchedAuditEventForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAuditEventForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAuditEventForecastStatus) -> bool {
        match self.status {
            BatchedAuditEventForecastStatus::Planned => {
                return next == BatchedAuditEventForecastStatus::Scheduled;
            }
            BatchedAuditEventForecastStatus::Scheduled => {
                return next == BatchedAuditEventForecastStatus::Completed
                    || next == BatchedAuditEventForecastStatus::Cancelled;
            }
            BatchedAuditEventForecastStatus::InProgress => {
                match next {
                    BatchedAuditEventForecastStatus::Completed | BatchedAuditEventForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAuditEventForecastStatus::Completed => {
                match next {
                    BatchedAuditEventForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedAuditEventForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched audit event forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAuditEventForecast {
        BatchedAuditEventForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            capacity: self.capacity,
            grace_days: self.grace_days,
            approved: self.approved,
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            pinned: self.pinned,
            namespace: self.namespace.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched audit event forecast in status planned.
pub fn new_batched_audit_event_forecast(code: &str, name: &str) -> BatchedAuditEventForecast {
    BatchedAuditEventForecast {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        capacity: 0,
        grace_days: 0,
        approved: false,
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        pinned: false,
        namespace: "".to_string(),
        cipher: "".to_string(),
        status: BatchedAuditEventForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_audit_event_forecast("code-134", "name-641");
        assert_eq!(item.status, BatchedAuditEventForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_batched_audit_event_forecast("code-134", "name-641");
        if item.move_to(BatchedAuditEventForecastStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAuditEventForecastStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_audit_event_forecast("code-134", "name-641");
        item.status = BatchedAuditEventForecastStatus::Cancelled;
        assert!(
            item.move_to(BatchedAuditEventForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
