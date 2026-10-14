use super::*;

use acme_kit::text;

/// Where a action item digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ActionItemDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ActionItemDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemDigest {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub sequence: i64,
    pub contact_email: String,
    pub amount: f64,
    pub fingerprint: String,
    pub status: ActionItemDigestStatus,
}

impl ActionItemDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemDigestStatus) -> bool {
        match self.status {
            ActionItemDigestStatus::Created => {
                match next {
                    ActionItemDigestStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemDigestStatus::Validated => {
                match next {
                    ActionItemDigestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemDigestStatus::Published => {
                return next == ActionItemDigestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.sequence as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemDigest {
        ActionItemDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            sequence: self.sequence,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a action item digest in status created.
pub fn new_action_item_digest(code: &str, title: &str) -> ActionItemDigest {
    ActionItemDigest {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        sequence: 0,
        contact_email: "".to_string(),
        amount: 0.0,
        fingerprint: "".to_string(),
        status: ActionItemDigestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_action_item_digest("code-919", "title-166");
        assert_eq!(item.status, ActionItemDigestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_action_item_digest("code-919", "title-166");
        if item.move_to(ActionItemDigestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemDigestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_action_item_digest("code-919", "title-166");
        item.status = ActionItemDigestStatus::Retired;
        assert!(
            item.move_to(ActionItemDigestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
