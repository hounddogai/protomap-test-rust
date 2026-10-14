use super::*;

use acme_kit::text;

/// Where a batched slo slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSloSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedSloSlotStatus {
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

impl std::fmt::Display for BatchedSloSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched slo slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloSlot {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub locale: String,
    pub cost: f64,
    pub source: String,
    pub scope: String,
    pub label: String,
    pub visible: bool,
    pub phone: String,
    pub status: BatchedSloSlotStatus,
}

impl BatchedSloSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSloSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSloSlotStatus) -> bool {
        match self.status {
            BatchedSloSlotStatus::Planned => {
                return next == BatchedSloSlotStatus::InProgress || next == BatchedSloSlotStatus::Completed;
            }
            BatchedSloSlotStatus::Scheduled => {
                match next {
                    BatchedSloSlotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSloSlotStatus::InProgress => {
                return next == BatchedSloSlotStatus::Completed || next == BatchedSloSlotStatus::Cancelled;
            }
            BatchedSloSlotStatus::Completed => {
                match next {
                    BatchedSloSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedSloSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched slo slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> BatchedSloSlot {
        BatchedSloSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            cost: self.cost,
            source: self.source.clone(),
            scope: self.scope.clone(),
            label: self.label.clone(),
            visible: self.visible,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched slo slot in status planned.
pub fn new_batched_slo_slot(code: &str, title: &str) -> BatchedSloSlot {
    BatchedSloSlot {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        locale: "".to_string(),
        cost: 0.0,
        source: "".to_string(),
        scope: "".to_string(),
        label: "".to_string(),
        visible: false,
        phone: "".to_string(),
        status: BatchedSloSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_slo_slot("code-819", "title-624");
        assert_eq!(item.status, BatchedSloSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_batched_slo_slot("code-819", "title-624");
        if item.move_to(BatchedSloSlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSloSlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_slo_slot("code-819", "title-624");
        item.status = BatchedSloSlotStatus::Cancelled;
        assert!(
            item.move_to(BatchedSloSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
