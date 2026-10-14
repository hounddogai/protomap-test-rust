use super::*;

use acme_kit::text;

/// Where a action item request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ActionItemRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ActionItemRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemRequest {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub version: i64,
    pub expires_at: i64,
    pub maximum: f64,
    pub minimum: f64,
    pub status: ActionItemRequestStatus,
}

impl ActionItemRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemRequestStatus) -> bool {
        match self.status {
            ActionItemRequestStatus::Proposed => {
                match next {
                    ActionItemRequestStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemRequestStatus::Approved => {
                return next == ActionItemRequestStatus::Rejected || next == ActionItemRequestStatus::Withdrawn;
            }
            ActionItemRequestStatus::Rejected => {
                return next == ActionItemRequestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.version as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemRequest {
        ActionItemRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            version: self.version,
            expires_at: self.expires_at,
            maximum: self.maximum,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a action item request in status proposed.
pub fn new_action_item_request(code: &str, label: &str) -> ActionItemRequest {
    ActionItemRequest {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        version: 0,
        expires_at: 0,
        maximum: 0.0,
        minimum: 0.0,
        status: ActionItemRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_action_item_request("code-439", "label-222");
        assert_eq!(item.status, ActionItemRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_action_item_request("code-439", "label-222");
        if item.move_to(ActionItemRequestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemRequestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_action_item_request("code-439", "label-222");
        item.status = ActionItemRequestStatus::Withdrawn;
        assert!(
            item.move_to(ActionItemRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
