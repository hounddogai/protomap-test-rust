use super::*;

use acme_kit::text;

/// Where a action item audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemAuditStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ActionItemAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ActionItemAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item audit as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAudit {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub name: String,
    pub locale: String,
    pub rotation_days: i64,
    pub cluster: String,
    pub status: ActionItemAuditStatus,
}

impl ActionItemAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemAuditStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemAuditStatus) -> bool {
        match self.status {
            ActionItemAuditStatus::Created => {
                match next {
                    ActionItemAuditStatus::Validated | ActionItemAuditStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemAuditStatus::Validated => {
                match next {
                    ActionItemAuditStatus::Published | ActionItemAuditStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemAuditStatus::Published => {
                match next {
                    ActionItemAuditStatus::Retired => {
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

    pub fn move_to(&mut self, next: ActionItemAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item audit", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemAudit {
        ActionItemAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            name: self.name.clone(),
            locale: self.locale.clone(),
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a action item audit in status created.
pub fn new_action_item_audit(code: &str, title: &str) -> ActionItemAudit {
    ActionItemAudit {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        name: "".to_string(),
        locale: "".to_string(),
        rotation_days: 0,
        cluster: "".to_string(),
        status: ActionItemAuditStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_action_item_audit("code-145", "title-968");
        assert_eq!(item.status, ActionItemAuditStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_action_item_audit("code-145", "title-968");
        if item.move_to(ActionItemAuditStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemAuditStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_action_item_audit("code-145", "title-968");
        item.status = ActionItemAuditStatus::Retired;
        assert!(
            item.move_to(ActionItemAuditStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
