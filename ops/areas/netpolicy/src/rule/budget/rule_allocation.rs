use super::*;

use acme_kit::text;

/// Where a rule allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RuleAllocationStatus {
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

impl std::fmt::Display for RuleAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAllocation {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub ratio: f64,
    pub owner_id: String,
    pub notes: String,
    pub maximum: f64,
    pub rate: f64,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub status: RuleAllocationStatus,
}

impl RuleAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleAllocationStatus) -> bool {
        match self.status {
            RuleAllocationStatus::New => {
                return next == RuleAllocationStatus::Running;
            }
            RuleAllocationStatus::Queued => {
                return next == RuleAllocationStatus::Done || next == RuleAllocationStatus::Failed;
            }
            RuleAllocationStatus::Running => {
                match next {
                    RuleAllocationStatus::Done | RuleAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleAllocationStatus::Done => {
                return next == RuleAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule allocation {} cannot move from {} to {}",
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleAllocation {
        RuleAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            notes: self.notes.clone(),
            maximum: self.maximum,
            rate: self.rate,
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule allocation in status new.
pub fn new_rule_allocation(code: &str, title: &str) -> RuleAllocation {
    RuleAllocation {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        ratio: 0.0,
        owner_id: "".to_string(),
        notes: "".to_string(),
        maximum: 0.0,
        rate: 0.0,
        created_at: 0,
        duration_minutes: 0,
        status: RuleAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rule_allocation("code-468", "title-2");
        assert_eq!(item.status, RuleAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rule_allocation("code-468", "title-2");
        if item.move_to(RuleAllocationStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleAllocationStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rule_allocation("code-468", "title-2");
        item.status = RuleAllocationStatus::Failed;
        assert!(
            item.move_to(RuleAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
