use super::*;

use acme_kit::text;

/// Where a weighted action item draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedActionItemDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedActionItemDraftStatus {
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

impl std::fmt::Display for WeightedActionItemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted action item draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemDraft {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub active: bool,
    pub priority: i64,
    pub created_at: i64,
    pub ends_at: i64,
    pub capacity: i64,
    pub status: WeightedActionItemDraftStatus,
}

impl WeightedActionItemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedActionItemDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedActionItemDraftStatus) -> bool {
        match self.status {
            WeightedActionItemDraftStatus::Planned => {
                match next {
                    WeightedActionItemDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedActionItemDraftStatus::Scheduled => {
                match next {
                    WeightedActionItemDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedActionItemDraftStatus::InProgress => {
                match next {
                    WeightedActionItemDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedActionItemDraftStatus::Completed => {
                return next == WeightedActionItemDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedActionItemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted action item draft {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "weighted action item draft", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ends_at as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedActionItemDraft {
        WeightedActionItemDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            active: self.active,
            priority: self.priority,
            created_at: self.created_at,
            ends_at: self.ends_at,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted action item draft in status planned.
pub fn new_weighted_action_item_draft(code: &str, name: &str) -> WeightedActionItemDraft {
    WeightedActionItemDraft {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        active: false,
        priority: 0,
        created_at: 0,
        ends_at: 0,
        capacity: 0,
        status: WeightedActionItemDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_action_item_draft("code-647", "name-403");
        assert_eq!(item.status, WeightedActionItemDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_action_item_draft("code-647", "name-403");
        if item.move_to(WeightedActionItemDraftStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedActionItemDraftStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_action_item_draft("code-647", "name-403");
        item.status = WeightedActionItemDraftStatus::Cancelled;
        assert!(
            item.move_to(WeightedActionItemDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
