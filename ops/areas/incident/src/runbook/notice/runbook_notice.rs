use super::*;

use acme_kit::text;

/// Where a runbook notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RunbookNoticeStatus {
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

impl std::fmt::Display for RunbookNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookNotice {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub cost: f64,
    pub created_at: i64,
    pub category: String,
    pub notes: String,
    pub label: String,
    pub rotation_days: i64,
    pub ratio: f64,
    pub approved: bool,
    pub status: RunbookNoticeStatus,
}

impl RunbookNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookNoticeStatus) -> bool {
        match self.status {
            RunbookNoticeStatus::Idle => {
                return next == RunbookNoticeStatus::Draining;
            }
            RunbookNoticeStatus::Warming => {
                match next {
                    RunbookNoticeStatus::Ready | RunbookNoticeStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookNoticeStatus::Ready => {
                return next == RunbookNoticeStatus::Draining || next == RunbookNoticeStatus::Stopped;
            }
            RunbookNoticeStatus::Draining => {
                return next == RunbookNoticeStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook notice {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.cost * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookNotice {
        RunbookNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            cost: self.cost,
            created_at: self.created_at,
            category: self.category.clone(),
            notes: self.notes.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a runbook notice in status idle.
pub fn new_runbook_notice(code: &str, name: &str) -> RunbookNotice {
    RunbookNotice {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        cost: 0.0,
        created_at: 0,
        category: "".to_string(),
        notes: "".to_string(),
        label: "".to_string(),
        rotation_days: 0,
        ratio: 0.0,
        approved: false,
        status: RunbookNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_runbook_notice("code-346", "name-557");
        assert_eq!(item.status, RunbookNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_runbook_notice("code-346", "name-557");
        if item.move_to(RunbookNoticeStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookNoticeStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_runbook_notice("code-346", "name-557");
        item.status = RunbookNoticeStatus::Stopped;
        assert!(
            item.move_to(RunbookNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
