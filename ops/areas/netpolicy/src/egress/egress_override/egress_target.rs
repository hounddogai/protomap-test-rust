use super::*;

use acme_kit::text;

/// Where a egress target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EgressTargetStatus {
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

impl std::fmt::Display for EgressTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressTarget {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub audience: String,
    pub priority: i64,
    pub unit: String,
    pub visible: bool,
    pub owner_id: String,
    pub summary: String,
    pub status: EgressTargetStatus,
}

impl EgressTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressTargetStatus) -> bool {
        match self.status {
            EgressTargetStatus::Planned => {
                match next {
                    EgressTargetStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressTargetStatus::Scheduled => {
                match next {
                    EgressTargetStatus::Completed | EgressTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressTargetStatus::InProgress => {
                match next {
                    EgressTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressTargetStatus::Completed => {
                match next {
                    EgressTargetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EgressTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> EgressTarget {
        EgressTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            audience: self.audience.clone(),
            priority: self.priority,
            unit: self.unit.clone(),
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a egress target in status planned.
pub fn new_egress_target(code: &str, name: &str) -> EgressTarget {
    EgressTarget {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        audience: "".to_string(),
        priority: 0,
        unit: "".to_string(),
        visible: false,
        owner_id: "".to_string(),
        summary: "".to_string(),
        status: EgressTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_egress_target("code-172", "name-382");
        assert_eq!(item.status, EgressTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_egress_target("code-172", "name-382");
        if item.move_to(EgressTargetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressTargetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_egress_target("code-172", "name-382");
        item.status = EgressTargetStatus::Cancelled;
        assert!(
            item.move_to(EgressTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
