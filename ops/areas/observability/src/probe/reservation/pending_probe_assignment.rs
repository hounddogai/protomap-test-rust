use super::*;

use acme_kit::text;

/// Where a pending probe assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingProbeAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingProbeAssignmentStatus {
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

impl std::fmt::Display for PendingProbeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending probe assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeAssignment {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub cost: f64,
    pub issuer: String,
    pub rate: f64,
    pub approved: bool,
    pub created_at: i64,
    pub title: String,
    pub archived: bool,
    pub status: PendingProbeAssignmentStatus,
}

impl PendingProbeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingProbeAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingProbeAssignmentStatus) -> bool {
        match self.status {
            PendingProbeAssignmentStatus::Idle => {
                match next {
                    PendingProbeAssignmentStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingProbeAssignmentStatus::Warming => {
                match next {
                    PendingProbeAssignmentStatus::Ready | PendingProbeAssignmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingProbeAssignmentStatus::Ready => {
                return next == PendingProbeAssignmentStatus::Draining;
            }
            PendingProbeAssignmentStatus::Draining => {
                match next {
                    PendingProbeAssignmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingProbeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending probe assignment {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingProbeAssignment {
        PendingProbeAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            cost: self.cost,
            issuer: self.issuer.clone(),
            rate: self.rate,
            approved: self.approved,
            created_at: self.created_at,
            title: self.title.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending probe assignment in status idle.
pub fn new_pending_probe_assignment(code: &str, name: &str) -> PendingProbeAssignment {
    PendingProbeAssignment {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        cost: 0.0,
        issuer: "".to_string(),
        rate: 0.0,
        approved: false,
        created_at: 0,
        title: "".to_string(),
        archived: false,
        status: PendingProbeAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_probe_assignment("code-605", "name-69");
        assert_eq!(item.status, PendingProbeAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_probe_assignment("code-605", "name-69");
        if item.move_to(PendingProbeAssignmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingProbeAssignmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_probe_assignment("code-605", "name-69");
        item.status = PendingProbeAssignmentStatus::Stopped;
        assert!(
            item.move_to(PendingProbeAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
