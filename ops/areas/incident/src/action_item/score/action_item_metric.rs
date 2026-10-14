use super::*;

use acme_kit::text;

/// Where a action item metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemMetricStatus {
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

impl std::fmt::Display for ActionItemMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMetric {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub weight: f64,
    pub expires_at: i64,
    pub amount: f64,
    pub owner_id: String,
    pub status: ActionItemMetricStatus,
}

impl ActionItemMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemMetricStatus) -> bool {
        match self.status {
            ActionItemMetricStatus::Draft => {
                match next {
                    ActionItemMetricStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemMetricStatus::Pending => {
                match next {
                    ActionItemMetricStatus::Active | ActionItemMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemMetricStatus::Active => {
                match next {
                    ActionItemMetricStatus::Suspended | ActionItemMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemMetricStatus::Suspended => {
                return next == ActionItemMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.expires_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemMetric {
        ActionItemMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            weight: self.weight,
            expires_at: self.expires_at,
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a action item metric in status draft.
pub fn new_action_item_metric(code: &str, label: &str) -> ActionItemMetric {
    ActionItemMetric {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        weight: 0.0,
        expires_at: 0,
        amount: 0.0,
        owner_id: "".to_string(),
        status: ActionItemMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_action_item_metric("code-276", "label-427");
        assert_eq!(item.status, ActionItemMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_action_item_metric("code-276", "label-427");
        if item.move_to(ActionItemMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_action_item_metric("code-276", "label-427");
        item.status = ActionItemMetricStatus::Closed;
        assert!(
            item.move_to(ActionItemMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
