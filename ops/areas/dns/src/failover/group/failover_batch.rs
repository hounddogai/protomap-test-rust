use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a failover batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FailoverBatchStatus {
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

impl std::fmt::Display for FailoverBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBatch {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub host: String,
    pub quantity: i64,
    pub sequence: i64,
    pub discount: f64,
    pub created_at: i64,
    pub owner_id: String,
    pub floor_value: f64,
    pub status: FailoverBatchStatus,
}

impl FailoverBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverBatchStatus) -> bool {
        match self.status {
            FailoverBatchStatus::Planned => {
                match next {
                    FailoverBatchStatus::Scheduled | FailoverBatchStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverBatchStatus::Scheduled => {
                match next {
                    FailoverBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverBatchStatus::InProgress => {
                return next == FailoverBatchStatus::Completed || next == FailoverBatchStatus::Cancelled;
            }
            FailoverBatchStatus::Completed => {
                match next {
                    FailoverBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FailoverBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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
        ((self.floor_value - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverBatch {
        FailoverBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            host: self.host.clone(),
            quantity: self.quantity,
            sequence: self.sequence,
            discount: self.discount,
            created_at: self.created_at,
            owner_id: self.owner_id.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover batch in status planned.
pub fn new_failover_batch(code: &str, label: &str) -> FailoverBatch {
    FailoverBatch {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        window_days: 0,
        host: "".to_string(),
        quantity: 0,
        sequence: 0,
        discount: 0.0,
        created_at: 0,
        owner_id: "".to_string(),
        floor_value: 0.0,
        status: FailoverBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_failover_batch("code-924", "label-8");
        assert_eq!(item.status, FailoverBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_failover_batch("code-924", "label-8");
        if item.move_to(FailoverBatchStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverBatchStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_failover_batch("code-924", "label-8");
        item.status = FailoverBatchStatus::Cancelled;
        assert!(
            item.move_to(FailoverBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
