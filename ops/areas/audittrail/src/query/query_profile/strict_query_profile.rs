use super::*;

use acme_kit::text;

/// Where a strict query profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictQueryProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictQueryProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for StrictQueryProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict query profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryProfile {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub grace_days: i64,
    pub timezone: String,
    pub retries: i64,
    pub owner_id: String,
    pub attempts: i64,
    pub unit: String,
    pub status: StrictQueryProfileStatus,
}

impl StrictQueryProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictQueryProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictQueryProfileStatus) -> bool {
        match self.status {
            StrictQueryProfileStatus::New => {
                return next == StrictQueryProfileStatus::Done || next == StrictQueryProfileStatus::Failed;
            }
            StrictQueryProfileStatus::Queued => {
                return next == StrictQueryProfileStatus::Failed;
            }
            StrictQueryProfileStatus::Running => {
                match next {
                    StrictQueryProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQueryProfileStatus::Done => {
                match next {
                    StrictQueryProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictQueryProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict query profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict query profile", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.attempts as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictQueryProfile {
        StrictQueryProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            retries: self.retries,
            owner_id: self.owner_id.clone(),
            attempts: self.attempts,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict query profile in status new.
pub fn new_strict_query_profile(code: &str, name: &str) -> StrictQueryProfile {
    StrictQueryProfile {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        duration_minutes: 0,
        grace_days: 0,
        timezone: "".to_string(),
        retries: 0,
        owner_id: "".to_string(),
        attempts: 0,
        unit: "".to_string(),
        status: StrictQueryProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_query_profile("code-176", "name-859");
        assert_eq!(item.status, StrictQueryProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_strict_query_profile("code-176", "name-859");
        if item.move_to(StrictQueryProfileStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictQueryProfileStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_query_profile("code-176", "name-859");
        item.status = StrictQueryProfileStatus::Failed;
        assert!(
            item.move_to(StrictQueryProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
