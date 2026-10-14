use super::*;

use acme_kit::text;

/// Where a action item summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemSummaryStatus {
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

impl std::fmt::Display for ActionItemSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSummary {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub cipher: String,
    pub floor_value: f64,
    pub threshold: f64,
    pub status: ActionItemSummaryStatus,
}

impl ActionItemSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemSummaryStatus) -> bool {
        match self.status {
            ActionItemSummaryStatus::Draft => {
                return next == ActionItemSummaryStatus::Pending;
            }
            ActionItemSummaryStatus::Pending => {
                match next {
                    ActionItemSummaryStatus::Active | ActionItemSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSummaryStatus::Active => {
                match next {
                    ActionItemSummaryStatus::Suspended | ActionItemSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSummaryStatus::Suspended => {
                return next == ActionItemSummaryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.capacity as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemSummary {
        ActionItemSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            cipher: self.cipher.clone(),
            floor_value: self.floor_value,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a action item summary in status draft.
pub fn new_action_item_summary(code: &str, title: &str) -> ActionItemSummary {
    ActionItemSummary {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        cipher: "".to_string(),
        floor_value: 0.0,
        threshold: 0.0,
        status: ActionItemSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_action_item_summary("code-806", "title-5");
        assert_eq!(item.status, ActionItemSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_action_item_summary("code-806", "title-5");
        if item.move_to(ActionItemSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_action_item_summary("code-806", "title-5");
        item.status = ActionItemSummaryStatus::Closed;
        assert!(
            item.move_to(ActionItemSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
