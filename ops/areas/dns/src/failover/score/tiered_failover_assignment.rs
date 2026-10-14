use super::*;

use acme_kit::text;

/// Where a tiered failover assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFailoverAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredFailoverAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredFailoverAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered failover assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAssignment {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub sequence: i64,
    pub timezone: String,
    pub cost: f64,
    pub status: TieredFailoverAssignmentStatus,
}

impl TieredFailoverAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFailoverAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFailoverAssignmentStatus) -> bool {
        match self.status {
            TieredFailoverAssignmentStatus::Created => {
                match next {
                    TieredFailoverAssignmentStatus::Published | TieredFailoverAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFailoverAssignmentStatus::Validated => {
                match next {
                    TieredFailoverAssignmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFailoverAssignmentStatus::Published => {
                return next == TieredFailoverAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredFailoverAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered failover assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered failover assignment", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFailoverAssignment {
        TieredFailoverAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            sequence: self.sequence,
            timezone: self.timezone.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered failover assignment in status created.
pub fn new_tiered_failover_assignment(code: &str, name: &str) -> TieredFailoverAssignment {
    TieredFailoverAssignment {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        sequence: 0,
        timezone: "".to_string(),
        cost: 0.0,
        status: TieredFailoverAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_failover_assignment("code-811", "name-944");
        assert_eq!(item.status, TieredFailoverAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tiered_failover_assignment("code-811", "name-944");
        if item.move_to(TieredFailoverAssignmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFailoverAssignmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_failover_assignment("code-811", "name-944");
        item.status = TieredFailoverAssignmentStatus::Retired;
        assert!(
            item.move_to(TieredFailoverAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
