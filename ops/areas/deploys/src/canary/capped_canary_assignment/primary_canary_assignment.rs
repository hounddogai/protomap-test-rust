use super::*;

use acme_kit::text;

/// Where a primary canary assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCanaryAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryCanaryAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryCanaryAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary canary assignment as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignment {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub pinned: bool,
    pub approved: bool,
    pub limit: i64,
    pub ceiling: f64,
    pub rotation_days: i64,
    pub sequence: i64,
    pub status: PrimaryCanaryAssignmentStatus,
}

impl PrimaryCanaryAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCanaryAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCanaryAssignmentStatus) -> bool {
        match self.status {
            PrimaryCanaryAssignmentStatus::Created => {
                match next {
                    PrimaryCanaryAssignmentStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCanaryAssignmentStatus::Validated => {
                return next == PrimaryCanaryAssignmentStatus::Published;
            }
            PrimaryCanaryAssignmentStatus::Published => {
                match next {
                    PrimaryCanaryAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryCanaryAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary canary assignment {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ceiling - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryCanaryAssignment {
        PrimaryCanaryAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            pinned: self.pinned,
            approved: self.approved,
            limit: self.limit,
            ceiling: self.ceiling,
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary canary assignment in status created.
pub fn new_primary_canary_assignment(code: &str, name: &str) -> PrimaryCanaryAssignment {
    PrimaryCanaryAssignment {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        pinned: false,
        approved: false,
        limit: 0,
        ceiling: 0.0,
        rotation_days: 0,
        sequence: 0,
        status: PrimaryCanaryAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_canary_assignment("code-878", "name-102");
        assert_eq!(item.status, PrimaryCanaryAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_canary_assignment("code-878", "name-102");
        if item.move_to(PrimaryCanaryAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryCanaryAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_canary_assignment("code-878", "name-102");
        item.status = PrimaryCanaryAssignmentStatus::Retired;
        assert!(
            item.move_to(PrimaryCanaryAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
