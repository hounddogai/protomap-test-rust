use super::*;

use acme_kit::text;

/// Where a verifier schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl VerifierScheduleStatus {
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

impl std::fmt::Display for VerifierScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSchedule {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub host: String,
    pub country: String,
    pub ceiling: f64,
    pub ttl_seconds: i64,
    pub window_days: i64,
    pub pinned: bool,
    pub ends_at: i64,
    pub tenant_id: String,
    pub status: VerifierScheduleStatus,
}

impl VerifierSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierScheduleStatus) -> bool {
        match self.status {
            VerifierScheduleStatus::New => {
                return next == VerifierScheduleStatus::Queued || next == VerifierScheduleStatus::Failed;
            }
            VerifierScheduleStatus::Queued => {
                return next == VerifierScheduleStatus::Running;
            }
            VerifierScheduleStatus::Running => {
                return next == VerifierScheduleStatus::Done || next == VerifierScheduleStatus::Failed;
            }
            VerifierScheduleStatus::Done => {
                return next == VerifierScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier schedule", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierSchedule {
        VerifierSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            host: self.host.clone(),
            country: self.country.clone(),
            ceiling: self.ceiling,
            ttl_seconds: self.ttl_seconds,
            window_days: self.window_days,
            pinned: self.pinned,
            ends_at: self.ends_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a verifier schedule in status new.
pub fn new_verifier_schedule(code: &str, name: &str) -> VerifierSchedule {
    VerifierSchedule {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        host: "".to_string(),
        country: "".to_string(),
        ceiling: 0.0,
        ttl_seconds: 0,
        window_days: 0,
        pinned: false,
        ends_at: 0,
        tenant_id: "".to_string(),
        status: VerifierScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_verifier_schedule("code-858", "name-946");
        assert_eq!(item.status, VerifierScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_verifier_schedule("code-858", "name-946");
        if item.move_to(VerifierScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_verifier_schedule("code-858", "name-946");
        item.status = VerifierScheduleStatus::Failed;
        assert!(
            item.move_to(VerifierScheduleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
