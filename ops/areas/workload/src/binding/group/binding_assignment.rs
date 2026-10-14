use super::*;

use acme_kit::text;

/// Where a binding assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingAssignmentStatus {
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

impl std::fmt::Display for BindingAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAssignment {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub floor_value: f64,
    pub threshold: f64,
    pub capacity: i64,
    pub issuer: String,
    pub sequence: i64,
    pub fingerprint: String,
    pub version: i64,
    pub score: f64,
    pub status: BindingAssignmentStatus,
}

impl BindingAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingAssignmentStatus) -> bool {
        match self.status {
            BindingAssignmentStatus::Idle => {
                match next {
                    BindingAssignmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingAssignmentStatus::Warming => {
                return next == BindingAssignmentStatus::Ready;
            }
            BindingAssignmentStatus::Ready => {
                return next == BindingAssignmentStatus::Draining || next == BindingAssignmentStatus::Stopped;
            }
            BindingAssignmentStatus::Draining => {
                match next {
                    BindingAssignmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BindingAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "binding assignment", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.floor_value * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingAssignment {
        BindingAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            floor_value: self.floor_value,
            threshold: self.threshold,
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a binding assignment in status idle.
pub fn new_binding_assignment(code: &str, title: &str) -> BindingAssignment {
    BindingAssignment {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        floor_value: 0.0,
        threshold: 0.0,
        capacity: 0,
        issuer: "".to_string(),
        sequence: 0,
        fingerprint: "".to_string(),
        version: 0,
        score: 0.0,
        status: BindingAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_assignment("code-884", "title-612");
        assert_eq!(item.status, BindingAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_binding_assignment("code-884", "title-612");
        if item.move_to(BindingAssignmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingAssignmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_assignment("code-884", "title-612");
        item.status = BindingAssignmentStatus::Stopped;
        assert!(
            item.move_to(BindingAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
