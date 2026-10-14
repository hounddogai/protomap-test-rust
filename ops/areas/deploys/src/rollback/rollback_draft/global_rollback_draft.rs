use super::*;

use acme_kit::text;

/// Where a global rollback draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRollbackDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalRollbackDraftStatus {
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

impl std::fmt::Display for GlobalRollbackDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global rollback draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraft {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub ceiling: f64,
    pub summary: String,
    pub external_ref: String,
    pub rate: f64,
    pub checksum: String,
    pub status: GlobalRollbackDraftStatus,
}

impl GlobalRollbackDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRollbackDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRollbackDraftStatus) -> bool {
        match self.status {
            GlobalRollbackDraftStatus::Idle => {
                match next {
                    GlobalRollbackDraftStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRollbackDraftStatus::Warming => {
                return next == GlobalRollbackDraftStatus::Draining || next == GlobalRollbackDraftStatus::Stopped;
            }
            GlobalRollbackDraftStatus::Ready => {
                match next {
                    GlobalRollbackDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRollbackDraftStatus::Draining => {
                return next == GlobalRollbackDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRollbackDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global rollback draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.ceiling * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRollbackDraft {
        GlobalRollbackDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            ceiling: self.ceiling,
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global rollback draft in status idle.
pub fn new_global_rollback_draft(code: &str, title: &str) -> GlobalRollbackDraft {
    GlobalRollbackDraft {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        ceiling: 0.0,
        summary: "".to_string(),
        external_ref: "".to_string(),
        rate: 0.0,
        checksum: "".to_string(),
        status: GlobalRollbackDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_rollback_draft("code-680", "title-857");
        assert_eq!(item.status, GlobalRollbackDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_global_rollback_draft("code-680", "title-857");
        if item.move_to(GlobalRollbackDraftStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRollbackDraftStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_rollback_draft("code-680", "title-857");
        item.status = GlobalRollbackDraftStatus::Stopped;
        assert!(
            item.move_to(GlobalRollbackDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
