use super::*;

use acme_kit::text;

/// Where a tiered flow log assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFlowLogAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredFlowLogAssignmentStatus {
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

impl std::fmt::Display for TieredFlowLogAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered flow log assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogAssignment {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub discount: f64,
    pub created_at: i64,
    pub category: String,
    pub principal: String,
    pub external_ref: String,
    pub status: TieredFlowLogAssignmentStatus,
}

impl TieredFlowLogAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFlowLogAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFlowLogAssignmentStatus) -> bool {
        match self.status {
            TieredFlowLogAssignmentStatus::Planned => {
                match next {
                    TieredFlowLogAssignmentStatus::Scheduled | TieredFlowLogAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogAssignmentStatus::Scheduled => {
                return next == TieredFlowLogAssignmentStatus::Completed
                    || next == TieredFlowLogAssignmentStatus::Cancelled;
            }
            TieredFlowLogAssignmentStatus::InProgress => {
                match next {
                    TieredFlowLogAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogAssignmentStatus::Completed => {
                return next == TieredFlowLogAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredFlowLogAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered flow log assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFlowLogAssignment {
        TieredFlowLogAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            discount: self.discount,
            created_at: self.created_at,
            category: self.category.clone(),
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered flow log assignment in status planned.
pub fn new_tiered_flow_log_assignment(code: &str, name: &str) -> TieredFlowLogAssignment {
    TieredFlowLogAssignment {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        duration_minutes: 0,
        burst: 0,
        discount: 0.0,
        created_at: 0,
        category: "".to_string(),
        principal: "".to_string(),
        external_ref: "".to_string(),
        status: TieredFlowLogAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_flow_log_assignment("code-925", "name-813");
        assert_eq!(item.status, TieredFlowLogAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_tiered_flow_log_assignment("code-925", "name-813");
        if item.move_to(TieredFlowLogAssignmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFlowLogAssignmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_flow_log_assignment("code-925", "name-813");
        item.status = TieredFlowLogAssignmentStatus::Cancelled;
        assert!(
            item.move_to(TieredFlowLogAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
