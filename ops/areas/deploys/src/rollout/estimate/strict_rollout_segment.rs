use super::*;

use acme_kit::text;

/// Where a strict rollout segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRolloutSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictRolloutSegmentStatus {
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

impl std::fmt::Display for StrictRolloutSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict rollout segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegment {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub quantity: i64,
    pub port: i64,
    pub title: String,
    pub rotation_days: i64,
    pub sequence: i64,
    pub attempts: i64,
    pub budget: f64,
    pub host: String,
    pub status: StrictRolloutSegmentStatus,
}

impl StrictRolloutSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRolloutSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRolloutSegmentStatus) -> bool {
        match self.status {
            StrictRolloutSegmentStatus::Draft => {
                match next {
                    StrictRolloutSegmentStatus::Suspended | StrictRolloutSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutSegmentStatus::Pending => {
                match next {
                    StrictRolloutSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutSegmentStatus::Active => {
                return next == StrictRolloutSegmentStatus::Suspended || next == StrictRolloutSegmentStatus::Closed;
            }
            StrictRolloutSegmentStatus::Suspended => {
                match next {
                    StrictRolloutSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictRolloutSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rollout segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRolloutSegment {
        StrictRolloutSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            port: self.port,
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            attempts: self.attempts,
            budget: self.budget,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict rollout segment in status draft.
pub fn new_strict_rollout_segment(code: &str, name: &str) -> StrictRolloutSegment {
    StrictRolloutSegment {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        quantity: 0,
        port: 0,
        title: "".to_string(),
        rotation_days: 0,
        sequence: 0,
        attempts: 0,
        budget: 0.0,
        host: "".to_string(),
        status: StrictRolloutSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_rollout_segment("code-893", "name-743");
        assert_eq!(item.status, StrictRolloutSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_strict_rollout_segment("code-893", "name-743");
        if item.move_to(StrictRolloutSegmentStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRolloutSegmentStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_rollout_segment("code-893", "name-743");
        item.status = StrictRolloutSegmentStatus::Closed;
        assert!(
            item.move_to(StrictRolloutSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
