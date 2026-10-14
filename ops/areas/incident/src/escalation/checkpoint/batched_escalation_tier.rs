use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched escalation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedEscalationTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedEscalationTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for BatchedEscalationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched escalation tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationTier {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub updated_at: i64,
    pub ceiling: f64,
    pub maximum: f64,
    pub minimum: f64,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub starts_at: i64,
    pub key_id: String,
    pub status: BatchedEscalationTierStatus,
}

impl BatchedEscalationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedEscalationTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedEscalationTierStatus) -> bool {
        match self.status {
            BatchedEscalationTierStatus::New => {
                match next {
                    BatchedEscalationTierStatus::Queued | BatchedEscalationTierStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedEscalationTierStatus::Queued => {
                return next == BatchedEscalationTierStatus::Done;
            }
            BatchedEscalationTierStatus::Running => {
                return next == BatchedEscalationTierStatus::Done;
            }
            BatchedEscalationTierStatus::Done => {
                return next == BatchedEscalationTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedEscalationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched escalation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
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

    pub fn weighted_total(&self) -> f64 {
        ((self.retries as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedEscalationTier {
        BatchedEscalationTier {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            updated_at: self.updated_at,
            ceiling: self.ceiling,
            maximum: self.maximum,
            minimum: self.minimum,
            rate: self.rate,
            attributes: self.attributes.clone(),
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched escalation tier in status new.
pub fn new_batched_escalation_tier(code: &str, label: &str) -> BatchedEscalationTier {
    BatchedEscalationTier {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        updated_at: 0,
        ceiling: 0.0,
        maximum: 0.0,
        minimum: 0.0,
        rate: 0.0,
        attributes: BTreeMap::new(),
        starts_at: 0,
        key_id: "".to_string(),
        status: BatchedEscalationTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_escalation_tier("code-969", "label-34");
        assert_eq!(item.status, BatchedEscalationTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_escalation_tier("code-969", "label-34");
        if item.move_to(BatchedEscalationTierStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedEscalationTierStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_escalation_tier("code-969", "label-34");
        item.status = BatchedEscalationTierStatus::Failed;
        assert!(
            item.move_to(BatchedEscalationTierStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
