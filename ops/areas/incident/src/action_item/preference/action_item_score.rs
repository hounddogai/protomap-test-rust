use super::*;

use acme_kit::text;

/// Where a action item score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemScoreStatus {
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

impl std::fmt::Display for ActionItemScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item score as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemScore {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub cluster: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub updated_at: i64,
    pub audience: String,
    pub status: ActionItemScoreStatus,
}

impl ActionItemScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemScoreStatus) -> bool {
        match self.status {
            ActionItemScoreStatus::Draft => {
                match next {
                    ActionItemScoreStatus::Active | ActionItemScoreStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemScoreStatus::Pending => {
                match next {
                    ActionItemScoreStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemScoreStatus::Active => {
                match next {
                    ActionItemScoreStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemScoreStatus::Suspended => {
                return next == ActionItemScoreStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item score {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.updated_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemScore {
        ActionItemScore {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            cluster: self.cluster.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a action item score in status draft.
pub fn new_action_item_score(code: &str, title: &str) -> ActionItemScore {
    ActionItemScore {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        cluster: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        updated_at: 0,
        audience: "".to_string(),
        status: ActionItemScoreStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_action_item_score("code-730", "title-793");
        assert_eq!(item.status, ActionItemScoreStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_action_item_score("code-730", "title-793");
        if item.move_to(ActionItemScoreStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemScoreStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_action_item_score("code-730", "title-793");
        item.status = ActionItemScoreStatus::Closed;
        assert!(
            item.move_to(ActionItemScoreStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
