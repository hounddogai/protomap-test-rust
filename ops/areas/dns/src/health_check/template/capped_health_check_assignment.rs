use super::*;

use acme_kit::text;

/// Where a capped health check assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedHealthCheckAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedHealthCheckAssignmentStatus {
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

impl std::fmt::Display for CappedHealthCheckAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped health check assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedHealthCheckAssignment {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub unit: String,
    pub pinned: bool,
    pub status: CappedHealthCheckAssignmentStatus,
}

impl CappedHealthCheckAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedHealthCheckAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedHealthCheckAssignmentStatus) -> bool {
        match self.status {
            CappedHealthCheckAssignmentStatus::Draft => {
                return next == CappedHealthCheckAssignmentStatus::Active
                    || next == CappedHealthCheckAssignmentStatus::Suspended;
            }
            CappedHealthCheckAssignmentStatus::Pending => {
                return next == CappedHealthCheckAssignmentStatus::Active
                    || next == CappedHealthCheckAssignmentStatus::Suspended;
            }
            CappedHealthCheckAssignmentStatus::Active => {
                match next {
                    CappedHealthCheckAssignmentStatus::Suspended | CappedHealthCheckAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedHealthCheckAssignmentStatus::Suspended => {
                return next == CappedHealthCheckAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedHealthCheckAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped health check assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped health check assignment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> CappedHealthCheckAssignment {
        CappedHealthCheckAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            unit: self.unit.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a capped health check assignment in status draft.
pub fn new_capped_health_check_assignment(code: &str, title: &str) -> CappedHealthCheckAssignment {
    CappedHealthCheckAssignment {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        unit: "".to_string(),
        pinned: false,
        status: CappedHealthCheckAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_health_check_assignment("code-767", "title-205");
        assert_eq!(item.status, CappedHealthCheckAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_capped_health_check_assignment("code-767", "title-205");
        if item.move_to(CappedHealthCheckAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedHealthCheckAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_health_check_assignment("code-767", "title-205");
        item.status = CappedHealthCheckAssignmentStatus::Closed;
        assert!(
            item.move_to(CappedHealthCheckAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
