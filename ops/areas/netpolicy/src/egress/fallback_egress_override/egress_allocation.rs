use super::*;

use acme_kit::text;

/// Where a egress allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressAllocationStatus {
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

impl std::fmt::Display for EgressAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressAllocation {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub unit: String,
    pub sequence: i64,
    pub notes: String,
    pub scope: String,
    pub issuer: String,
    pub budget: f64,
    pub burst: i64,
    pub status: EgressAllocationStatus,
}

impl EgressAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressAllocationStatus) -> bool {
        match self.status {
            EgressAllocationStatus::Draft => {
                return next == EgressAllocationStatus::Pending || next == EgressAllocationStatus::Active;
            }
            EgressAllocationStatus::Pending => {
                return next == EgressAllocationStatus::Active || next == EgressAllocationStatus::Suspended;
            }
            EgressAllocationStatus::Active => {
                match next {
                    EgressAllocationStatus::Suspended | EgressAllocationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressAllocationStatus::Suspended => {
                return next == EgressAllocationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "egress allocation", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.budget - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressAllocation {
        EgressAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            unit: self.unit.clone(),
            sequence: self.sequence,
            notes: self.notes.clone(),
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            budget: self.budget,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a egress allocation in status draft.
pub fn new_egress_allocation(code: &str, name: &str) -> EgressAllocation {
    EgressAllocation {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        unit: "".to_string(),
        sequence: 0,
        notes: "".to_string(),
        scope: "".to_string(),
        issuer: "".to_string(),
        budget: 0.0,
        burst: 0,
        status: EgressAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_allocation("code-578", "name-741");
        assert_eq!(item.status, EgressAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_allocation("code-578", "name-741");
        if item.move_to(EgressAllocationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressAllocationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_allocation("code-578", "name-741");
        item.status = EgressAllocationStatus::Closed;
        assert!(
            item.move_to(EgressAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
