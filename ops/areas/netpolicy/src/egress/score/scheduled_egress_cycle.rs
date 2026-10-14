use super::*;

use acme_kit::text;

/// Where a scheduled egress cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEgressCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledEgressCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledEgressCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled egress cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressCycle {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub fingerprint: String,
    pub visible: bool,
    pub key_id: String,
    pub amount: f64,
    pub status: ScheduledEgressCycleStatus,
}

impl ScheduledEgressCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEgressCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEgressCycleStatus) -> bool {
        match self.status {
            ScheduledEgressCycleStatus::Created => {
                match next {
                    ScheduledEgressCycleStatus::Validated | ScheduledEgressCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEgressCycleStatus::Validated => {
                match next {
                    ScheduledEgressCycleStatus::Published | ScheduledEgressCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEgressCycleStatus::Published => {
                return next == ScheduledEgressCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledEgressCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled egress cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ScheduledEgressCycle {
        ScheduledEgressCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            fingerprint: self.fingerprint.clone(),
            visible: self.visible,
            key_id: self.key_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled egress cycle in status created.
pub fn new_scheduled_egress_cycle(code: &str, name: &str) -> ScheduledEgressCycle {
    ScheduledEgressCycle {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        fingerprint: "".to_string(),
        visible: false,
        key_id: "".to_string(),
        amount: 0.0,
        status: ScheduledEgressCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_egress_cycle("code-1", "name-899");
        assert_eq!(item.status, ScheduledEgressCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_scheduled_egress_cycle("code-1", "name-899");
        if item.move_to(ScheduledEgressCycleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledEgressCycleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_egress_cycle("code-1", "name-899");
        item.status = ScheduledEgressCycleStatus::Retired;
        assert!(
            item.move_to(ScheduledEgressCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
