use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a freeze plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezePlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezePlan {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub description: String,
    pub cluster: String,
    pub floor_value: f64,
    pub priority: i64,
    pub status: FreezePlanStatus,
}

impl FreezePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezePlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezePlanStatus) -> bool {
        match self.status {
            FreezePlanStatus::Proposed => {
                match next {
                    FreezePlanStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezePlanStatus::Approved => {
                return next == FreezePlanStatus::Rejected;
            }
            FreezePlanStatus::Rejected => {
                return next == FreezePlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze plan", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_value(&self) -> f64 {
        (self.floor_value * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezePlan {
        FreezePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            description: self.description.clone(),
            cluster: self.cluster.clone(),
            floor_value: self.floor_value,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a freeze plan in status proposed.
pub fn new_freeze_plan(code: &str, name: &str) -> FreezePlan {
    FreezePlan {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        description: "".to_string(),
        cluster: "".to_string(),
        floor_value: 0.0,
        priority: 0,
        status: FreezePlanStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_freeze_plan("code-952", "name-540");
        assert_eq!(item.status, FreezePlanStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_freeze_plan("code-952", "name-540");
        if item.move_to(FreezePlanStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezePlanStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_freeze_plan("code-952", "name-540");
        item.status = FreezePlanStatus::Withdrawn;
        assert!(
            item.move_to(FreezePlanStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
