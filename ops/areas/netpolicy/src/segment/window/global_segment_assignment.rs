use super::*;

use acme_kit::text;

/// Where a global segment assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSegmentAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalSegmentAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalSegmentAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global segment assignment as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentAssignment {
    pub code: String,
    pub name: String,
    pub description: String,
    pub starts_at: i64,
    pub duration_minutes: i64,
    pub status: GlobalSegmentAssignmentStatus,
}

impl GlobalSegmentAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSegmentAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSegmentAssignmentStatus) -> bool {
        match self.status {
            GlobalSegmentAssignmentStatus::Created => {
                match next {
                    GlobalSegmentAssignmentStatus::Validated | GlobalSegmentAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentAssignmentStatus::Validated => {
                match next {
                    GlobalSegmentAssignmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentAssignmentStatus::Published => {
                return next == GlobalSegmentAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSegmentAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global segment assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSegmentAssignment {
        GlobalSegmentAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            starts_at: self.starts_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global segment assignment in status created.
pub fn new_global_segment_assignment(code: &str, name: &str) -> GlobalSegmentAssignment {
    GlobalSegmentAssignment {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        starts_at: 0,
        duration_minutes: 0,
        status: GlobalSegmentAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_segment_assignment("code-3", "name-761");
        assert_eq!(item.status, GlobalSegmentAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_global_segment_assignment("code-3", "name-761");
        if item.move_to(GlobalSegmentAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSegmentAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_segment_assignment("code-3", "name-761");
        item.status = GlobalSegmentAssignmentStatus::Retired;
        assert!(
            item.move_to(GlobalSegmentAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
