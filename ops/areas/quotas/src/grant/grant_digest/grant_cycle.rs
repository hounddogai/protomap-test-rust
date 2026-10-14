use super::*;

use acme_kit::text;

/// Where a grant cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantCycleStatus {
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

impl std::fmt::Display for GrantCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantCycle {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub created_at: i64,
    pub active: bool,
    pub external_ref: String,
    pub updated_at: i64,
    pub window_days: i64,
    pub reviewer: String,
    pub status: GrantCycleStatus,
}

impl GrantCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantCycleStatus) -> bool {
        match self.status {
            GrantCycleStatus::New => {
                return next == GrantCycleStatus::Running;
            }
            GrantCycleStatus::Queued => {
                return next == GrantCycleStatus::Running;
            }
            GrantCycleStatus::Running => {
                return next == GrantCycleStatus::Failed;
            }
            GrantCycleStatus::Done => {
                return next == GrantCycleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant cycle {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantCycle {
        GrantCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            created_at: self.created_at,
            active: self.active,
            external_ref: self.external_ref.clone(),
            updated_at: self.updated_at,
            window_days: self.window_days,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a grant cycle in status new.
pub fn new_grant_cycle(code: &str, name: &str) -> GrantCycle {
    GrantCycle {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        created_at: 0,
        active: false,
        external_ref: "".to_string(),
        updated_at: 0,
        window_days: 0,
        reviewer: "".to_string(),
        status: GrantCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_cycle("code-802", "name-903");
        assert_eq!(item.status, GrantCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_grant_cycle("code-802", "name-903");
        if item.move_to(GrantCycleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantCycleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_cycle("code-802", "name-903");
        item.status = GrantCycleStatus::Failed;
        assert!(item.move_to(GrantCycleStatus::New).is_err(), "expected move to fail");
    }
}
