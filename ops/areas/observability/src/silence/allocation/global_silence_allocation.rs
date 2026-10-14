use super::*;

use acme_kit::text;

/// Where a global silence allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSilenceAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalSilenceAllocationStatus {
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

impl std::fmt::Display for GlobalSilenceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global silence allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocation {
    pub code: String,
    pub label: String,
    pub description: String,
    pub scope: String,
    pub ends_at: i64,
    pub category: String,
    pub pinned: bool,
    pub status: GlobalSilenceAllocationStatus,
}

impl GlobalSilenceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSilenceAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSilenceAllocationStatus) -> bool {
        match self.status {
            GlobalSilenceAllocationStatus::New => {
                return next == GlobalSilenceAllocationStatus::Queued;
            }
            GlobalSilenceAllocationStatus::Queued => {
                return next == GlobalSilenceAllocationStatus::Done;
            }
            GlobalSilenceAllocationStatus::Running => {
                return next == GlobalSilenceAllocationStatus::Failed;
            }
            GlobalSilenceAllocationStatus::Done => {
                return next == GlobalSilenceAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSilenceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global silence allocation {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "global silence allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> GlobalSilenceAllocation {
        GlobalSilenceAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            scope: self.scope.clone(),
            ends_at: self.ends_at,
            category: self.category.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global silence allocation in status new.
pub fn new_global_silence_allocation(code: &str, label: &str) -> GlobalSilenceAllocation {
    GlobalSilenceAllocation {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        scope: "".to_string(),
        ends_at: 0,
        category: "".to_string(),
        pinned: false,
        status: GlobalSilenceAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_silence_allocation("code-933", "label-195");
        assert_eq!(item.status, GlobalSilenceAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_global_silence_allocation("code-933", "label-195");
        if item.move_to(GlobalSilenceAllocationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSilenceAllocationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_silence_allocation("code-933", "label-195");
        item.status = GlobalSilenceAllocationStatus::Failed;
        assert!(
            item.move_to(GlobalSilenceAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
