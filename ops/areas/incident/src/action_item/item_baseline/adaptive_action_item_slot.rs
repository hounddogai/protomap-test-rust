use super::*;

use acme_kit::text;

/// Where a adaptive action item slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveActionItemSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveActionItemSlotStatus {
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

impl std::fmt::Display for AdaptiveActionItemSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive action item slot as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveActionItemSlot {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub cost: f64,
    pub quantity: i64,
    pub region: String,
    pub scope: String,
    pub label: String,
    pub burst: i64,
    pub created_at: i64,
    pub status: AdaptiveActionItemSlotStatus,
}

impl AdaptiveActionItemSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveActionItemSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveActionItemSlotStatus) -> bool {
        match self.status {
            AdaptiveActionItemSlotStatus::Planned => {
                match next {
                    AdaptiveActionItemSlotStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveActionItemSlotStatus::Scheduled => {
                match next {
                    AdaptiveActionItemSlotStatus::Completed | AdaptiveActionItemSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveActionItemSlotStatus::InProgress => {
                return next == AdaptiveActionItemSlotStatus::Completed
                    || next == AdaptiveActionItemSlotStatus::Cancelled;
            }
            AdaptiveActionItemSlotStatus::Completed => {
                return next == AdaptiveActionItemSlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveActionItemSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive action item slot {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveActionItemSlot {
        AdaptiveActionItemSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            cost: self.cost,
            quantity: self.quantity,
            region: self.region.clone(),
            scope: self.scope.clone(),
            label: self.label.clone(),
            burst: self.burst,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a adaptive action item slot in status planned.
pub fn new_adaptive_action_item_slot(code: &str, title: &str) -> AdaptiveActionItemSlot {
    AdaptiveActionItemSlot {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        cost: 0.0,
        quantity: 0,
        region: "".to_string(),
        scope: "".to_string(),
        label: "".to_string(),
        burst: 0,
        created_at: 0,
        status: AdaptiveActionItemSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_action_item_slot("code-490", "title-436");
        assert_eq!(item.status, AdaptiveActionItemSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_adaptive_action_item_slot("code-490", "title-436");
        if item.move_to(AdaptiveActionItemSlotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveActionItemSlotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_action_item_slot("code-490", "title-436");
        item.status = AdaptiveActionItemSlotStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveActionItemSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
