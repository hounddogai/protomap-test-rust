use super::*;

use acme_kit::text;

/// Where a federation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FederationTargetStatus {
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

impl std::fmt::Display for FederationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationTarget {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub priority: i64,
    pub capacity: i64,
    pub status: FederationTargetStatus,
}

impl FederationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationTargetStatus) -> bool {
        match self.status {
            FederationTargetStatus::Planned => {
                match next {
                    FederationTargetStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationTargetStatus::Scheduled => {
                match next {
                    FederationTargetStatus::InProgress | FederationTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationTargetStatus::InProgress => {
                return next == FederationTargetStatus::Completed || next == FederationTargetStatus::Cancelled;
            }
            FederationTargetStatus::Completed => {
                match next {
                    FederationTargetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FederationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationTarget {
        FederationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            priority: self.priority,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a federation target in status planned.
pub fn new_federation_target(code: &str, name: &str) -> FederationTarget {
    FederationTarget {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        priority: 0,
        capacity: 0,
        status: FederationTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_federation_target("code-296", "name-405");
        assert_eq!(item.status, FederationTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_federation_target("code-296", "name-405");
        if item.move_to(FederationTargetStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationTargetStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_federation_target("code-296", "name-405");
        item.status = FederationTargetStatus::Cancelled;
        assert!(
            item.move_to(FederationTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
