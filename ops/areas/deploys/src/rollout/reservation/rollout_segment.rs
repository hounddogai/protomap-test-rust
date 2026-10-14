use super::*;

use acme_kit::text;

/// Where a rollout segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RolloutSegmentStatus {
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

impl std::fmt::Display for RolloutSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSegment {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub batch_size: i64,
    pub country: String,
    pub expires_at: i64,
    pub limit: i64,
    pub status: RolloutSegmentStatus,
}

impl RolloutSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutSegmentStatus) -> bool {
        match self.status {
            RolloutSegmentStatus::Planned => {
                return next == RolloutSegmentStatus::Scheduled || next == RolloutSegmentStatus::Completed;
            }
            RolloutSegmentStatus::Scheduled => {
                match next {
                    RolloutSegmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutSegmentStatus::InProgress => {
                match next {
                    RolloutSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutSegmentStatus::Completed => {
                match next {
                    RolloutSegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RolloutSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout segment {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.weight - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutSegment {
        RolloutSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            batch_size: self.batch_size,
            country: self.country.clone(),
            expires_at: self.expires_at,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rollout segment in status planned.
pub fn new_rollout_segment(code: &str, title: &str) -> RolloutSegment {
    RolloutSegment {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        batch_size: 0,
        country: "".to_string(),
        expires_at: 0,
        limit: 0,
        status: RolloutSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollout_segment("code-749", "title-690");
        assert_eq!(item.status, RolloutSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rollout_segment("code-749", "title-690");
        if item.move_to(RolloutSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollout_segment("code-749", "title-690");
        item.status = RolloutSegmentStatus::Cancelled;
        assert!(
            item.move_to(RolloutSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
