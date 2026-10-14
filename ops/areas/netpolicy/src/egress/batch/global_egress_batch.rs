use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a global egress batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEgressBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalEgressBatchStatus {
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

impl std::fmt::Display for GlobalEgressBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global egress batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressBatch {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub owner_id: String,
    pub description: String,
    pub ends_at: i64,
    pub active: bool,
    pub floor_value: f64,
    pub attributes: BTreeMap<String, String>,
    pub external_ref: String,
    pub cost: f64,
    pub status: GlobalEgressBatchStatus,
}

impl GlobalEgressBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEgressBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEgressBatchStatus) -> bool {
        match self.status {
            GlobalEgressBatchStatus::New => {
                return next == GlobalEgressBatchStatus::Done || next == GlobalEgressBatchStatus::Failed;
            }
            GlobalEgressBatchStatus::Queued => {
                match next {
                    GlobalEgressBatchStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEgressBatchStatus::Running => {
                return next == GlobalEgressBatchStatus::Done || next == GlobalEgressBatchStatus::Failed;
            }
            GlobalEgressBatchStatus::Done => {
                match next {
                    GlobalEgressBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalEgressBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global egress batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
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

    pub fn adjusted_total(&self) -> f64 {
        (self.maximum * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEgressBatch {
        GlobalEgressBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            owner_id: self.owner_id.clone(),
            description: self.description.clone(),
            ends_at: self.ends_at,
            active: self.active,
            floor_value: self.floor_value,
            attributes: self.attributes.clone(),
            external_ref: self.external_ref.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global egress batch in status new.
pub fn new_global_egress_batch(code: &str, label: &str) -> GlobalEgressBatch {
    GlobalEgressBatch {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        owner_id: "".to_string(),
        description: "".to_string(),
        ends_at: 0,
        active: false,
        floor_value: 0.0,
        attributes: BTreeMap::new(),
        external_ref: "".to_string(),
        cost: 0.0,
        status: GlobalEgressBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_egress_batch("code-153", "label-177");
        assert_eq!(item.status, GlobalEgressBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_global_egress_batch("code-153", "label-177");
        if item.move_to(GlobalEgressBatchStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalEgressBatchStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_egress_batch("code-153", "label-177");
        item.status = GlobalEgressBatchStatus::Failed;
        assert!(
            item.move_to(GlobalEgressBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
