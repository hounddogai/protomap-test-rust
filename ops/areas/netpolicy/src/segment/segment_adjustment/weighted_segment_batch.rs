use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted segment batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSegmentBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedSegmentBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for WeightedSegmentBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted segment batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentBatch {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub visible: bool,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub starts_at: i64,
    pub principal: String,
    pub external_ref: String,
    pub maximum: f64,
    pub reviewer: String,
    pub status: WeightedSegmentBatchStatus,
}

impl WeightedSegmentBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSegmentBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSegmentBatchStatus) -> bool {
        match self.status {
            WeightedSegmentBatchStatus::Draft => {
                match next {
                    WeightedSegmentBatchStatus::Suspended | WeightedSegmentBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentBatchStatus::Pending => {
                return next == WeightedSegmentBatchStatus::Active;
            }
            WeightedSegmentBatchStatus::Active => {
                return next == WeightedSegmentBatchStatus::Suspended || next == WeightedSegmentBatchStatus::Closed;
            }
            WeightedSegmentBatchStatus::Suspended => {
                match next {
                    WeightedSegmentBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedSegmentBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted segment batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted segment batch", self.code)
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
        ((self.rotation_days as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSegmentBatch {
        WeightedSegmentBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            visible: self.visible,
            attributes: self.attributes.clone(),
            amount: self.amount,
            starts_at: self.starts_at,
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted segment batch in status draft.
pub fn new_weighted_segment_batch(code: &str, label: &str) -> WeightedSegmentBatch {
    WeightedSegmentBatch {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        visible: false,
        attributes: BTreeMap::new(),
        amount: 0.0,
        starts_at: 0,
        principal: "".to_string(),
        external_ref: "".to_string(),
        maximum: 0.0,
        reviewer: "".to_string(),
        status: WeightedSegmentBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_segment_batch("code-638", "label-861");
        assert_eq!(item.status, WeightedSegmentBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_weighted_segment_batch("code-638", "label-861");
        if item.move_to(WeightedSegmentBatchStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSegmentBatchStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_segment_batch("code-638", "label-861");
        item.status = WeightedSegmentBatchStatus::Closed;
        assert!(
            item.move_to(WeightedSegmentBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
