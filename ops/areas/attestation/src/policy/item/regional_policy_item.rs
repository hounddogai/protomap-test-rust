use super::*;

use acme_kit::text;

/// Where a regional policy item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPolicyItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalPolicyItemStatus {
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

impl std::fmt::Display for RegionalPolicyItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional policy item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyItem {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub discount: f64,
    pub scope: String,
    pub created_at: i64,
    pub reviewer: String,
    pub issuer: String,
    pub visible: bool,
    pub grace_days: i64,
    pub status: RegionalPolicyItemStatus,
}

impl RegionalPolicyItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPolicyItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPolicyItemStatus) -> bool {
        match self.status {
            RegionalPolicyItemStatus::Planned => {
                match next {
                    RegionalPolicyItemStatus::Completed | RegionalPolicyItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPolicyItemStatus::Scheduled => {
                return next == RegionalPolicyItemStatus::InProgress || next == RegionalPolicyItemStatus::Completed;
            }
            RegionalPolicyItemStatus::InProgress => {
                match next {
                    RegionalPolicyItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPolicyItemStatus::Completed => {
                return next == RegionalPolicyItemStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPolicyItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional policy item {} cannot move from {} to {}",
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
        format!("{} {}", "regional policy item", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.created_at as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPolicyItem {
        RegionalPolicyItem {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            discount: self.discount,
            scope: self.scope.clone(),
            created_at: self.created_at,
            reviewer: self.reviewer.clone(),
            issuer: self.issuer.clone(),
            visible: self.visible,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional policy item in status planned.
pub fn new_regional_policy_item(code: &str, name: &str) -> RegionalPolicyItem {
    RegionalPolicyItem {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        discount: 0.0,
        scope: "".to_string(),
        created_at: 0,
        reviewer: "".to_string(),
        issuer: "".to_string(),
        visible: false,
        grace_days: 0,
        status: RegionalPolicyItemStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_policy_item("code-218", "name-765");
        assert_eq!(item.status, RegionalPolicyItemStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_regional_policy_item("code-218", "name-765");
        if item.move_to(RegionalPolicyItemStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPolicyItemStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_policy_item("code-218", "name-765");
        item.status = RegionalPolicyItemStatus::Cancelled;
        assert!(
            item.move_to(RegionalPolicyItemStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
