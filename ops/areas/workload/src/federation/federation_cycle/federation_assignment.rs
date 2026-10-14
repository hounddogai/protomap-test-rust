use super::*;

use acme_kit::text;

/// Where a federation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for FederationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation assignment as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAssignment {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub updated_at: i64,
    pub principal: String,
    pub maximum: f64,
    pub created_at: i64,
    pub retries: i64,
    pub duration_minutes: i64,
    pub scope: String,
    pub notes: String,
    pub status: FederationAssignmentStatus,
}

impl FederationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationAssignmentStatus) -> bool {
        match self.status {
            FederationAssignmentStatus::Idle => {
                match next {
                    FederationAssignmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationAssignmentStatus::Warming => {
                match next {
                    FederationAssignmentStatus::Draining | FederationAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationAssignmentStatus::Ready => {
                match next {
                    FederationAssignmentStatus::Draining | FederationAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationAssignmentStatus::Draining => {
                return next == FederationAssignmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationAssignment {
        FederationAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            maximum: self.maximum,
            created_at: self.created_at,
            retries: self.retries,
            duration_minutes: self.duration_minutes,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a federation assignment in status idle.
pub fn new_federation_assignment(code: &str, title: &str) -> FederationAssignment {
    FederationAssignment {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        updated_at: 0,
        principal: "".to_string(),
        maximum: 0.0,
        created_at: 0,
        retries: 0,
        duration_minutes: 0,
        scope: "".to_string(),
        notes: "".to_string(),
        status: FederationAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_federation_assignment("code-90", "title-613");
        assert_eq!(item.status, FederationAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_federation_assignment("code-90", "title-613");
        if item.move_to(FederationAssignmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationAssignmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_federation_assignment("code-90", "title-613");
        item.status = FederationAssignmentStatus::Stopped;
        assert!(
            item.move_to(FederationAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
