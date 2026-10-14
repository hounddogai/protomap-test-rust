use super::*;

use acme_kit::text;

/// Where a default egress milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEgressMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultEgressMilestoneStatus {
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

impl std::fmt::Display for DefaultEgressMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default egress milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressMilestone {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub visible: bool,
    pub key_id: String,
    pub revision: i64,
    pub floor_value: f64,
    pub status: DefaultEgressMilestoneStatus,
}

impl DefaultEgressMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEgressMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEgressMilestoneStatus) -> bool {
        match self.status {
            DefaultEgressMilestoneStatus::Draft => {
                match next {
                    DefaultEgressMilestoneStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEgressMilestoneStatus::Pending => {
                match next {
                    DefaultEgressMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEgressMilestoneStatus::Active => {
                return next == DefaultEgressMilestoneStatus::Suspended || next == DefaultEgressMilestoneStatus::Closed;
            }
            DefaultEgressMilestoneStatus::Suspended => {
                match next {
                    DefaultEgressMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultEgressMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default egress milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEgressMilestone {
        DefaultEgressMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            visible: self.visible,
            key_id: self.key_id.clone(),
            revision: self.revision,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default egress milestone in status draft.
pub fn new_default_egress_milestone(code: &str, name: &str) -> DefaultEgressMilestone {
    DefaultEgressMilestone {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        visible: false,
        key_id: "".to_string(),
        revision: 0,
        floor_value: 0.0,
        status: DefaultEgressMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_egress_milestone("code-469", "name-501");
        assert_eq!(item.status, DefaultEgressMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_default_egress_milestone("code-469", "name-501");
        if item.move_to(DefaultEgressMilestoneStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEgressMilestoneStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_egress_milestone("code-469", "name-501");
        item.status = DefaultEgressMilestoneStatus::Closed;
        assert!(
            item.move_to(DefaultEgressMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
