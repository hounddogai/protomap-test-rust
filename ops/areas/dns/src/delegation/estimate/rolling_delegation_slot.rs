use super::*;

use acme_kit::text;

/// Where a rolling delegation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDelegationSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingDelegationSlotStatus {
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

impl std::fmt::Display for RollingDelegationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling delegation slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlot {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub namespace: String,
    pub segment: String,
    pub archived: bool,
    pub status: RollingDelegationSlotStatus,
}

impl RollingDelegationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDelegationSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDelegationSlotStatus) -> bool {
        match self.status {
            RollingDelegationSlotStatus::Draft => {
                match next {
                    RollingDelegationSlotStatus::Pending | RollingDelegationSlotStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDelegationSlotStatus::Pending => {
                match next {
                    RollingDelegationSlotStatus::Active | RollingDelegationSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDelegationSlotStatus::Active => {
                return next == RollingDelegationSlotStatus::Suspended;
            }
            RollingDelegationSlotStatus::Suspended => {
                return next == RollingDelegationSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingDelegationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling delegation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling delegation slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> RollingDelegationSlot {
        RollingDelegationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling delegation slot in status draft.
pub fn new_rolling_delegation_slot(code: &str, name: &str) -> RollingDelegationSlot {
    RollingDelegationSlot {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        namespace: "".to_string(),
        segment: "".to_string(),
        archived: false,
        status: RollingDelegationSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_delegation_slot("code-330", "name-394");
        assert_eq!(item.status, RollingDelegationSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_delegation_slot("code-330", "name-394");
        if item.move_to(RollingDelegationSlotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDelegationSlotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_delegation_slot("code-330", "name-394");
        item.status = RollingDelegationSlotStatus::Closed;
        assert!(
            item.move_to(RollingDelegationSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
