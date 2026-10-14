use super::*;

use acme_kit::text;

/// Where a manual action item rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualActionItemRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualActionItemRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualActionItemRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual action item rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemRollup {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub principal: String,
    pub active: bool,
    pub tenant_id: String,
    pub contact_email: String,
    pub floor_value: f64,
    pub status: ManualActionItemRollupStatus,
}

impl ManualActionItemRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualActionItemRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualActionItemRollupStatus) -> bool {
        match self.status {
            ManualActionItemRollupStatus::Open => {
                match next {
                    ManualActionItemRollupStatus::Reopened | ManualActionItemRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualActionItemRollupStatus::InReview => {
                match next {
                    ManualActionItemRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualActionItemRollupStatus::Resolved => {
                match next {
                    ManualActionItemRollupStatus::Reopened | ManualActionItemRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualActionItemRollupStatus::Reopened => {
                return next == ManualActionItemRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualActionItemRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual action item rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.floor_value + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualActionItemRollup {
        ManualActionItemRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            principal: self.principal.clone(),
            active: self.active,
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual action item rollup in status open.
pub fn new_manual_action_item_rollup(code: &str, name: &str) -> ManualActionItemRollup {
    ManualActionItemRollup {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        principal: "".to_string(),
        active: false,
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        floor_value: 0.0,
        status: ManualActionItemRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_action_item_rollup("code-444", "name-555");
        assert_eq!(item.status, ManualActionItemRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_manual_action_item_rollup("code-444", "name-555");
        if item.move_to(ManualActionItemRollupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualActionItemRollupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_action_item_rollup("code-444", "name-555");
        item.status = ManualActionItemRollupStatus::Archived;
        assert!(
            item.move_to(ManualActionItemRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
