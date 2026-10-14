use super::*;

use acme_kit::text;

/// Where a global plan draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPlanDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalPlanDraftStatus {
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

impl std::fmt::Display for GlobalPlanDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global plan draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanDraft {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub floor_value: f64,
    pub cipher: String,
    pub reviewer: String,
    pub burst: i64,
    pub tenant_id: String,
    pub phone: String,
    pub expires_at: i64,
    pub window_days: i64,
    pub status: GlobalPlanDraftStatus,
}

impl GlobalPlanDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPlanDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPlanDraftStatus) -> bool {
        match self.status {
            GlobalPlanDraftStatus::New => {
                match next {
                    GlobalPlanDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanDraftStatus::Queued => {
                return next == GlobalPlanDraftStatus::Running || next == GlobalPlanDraftStatus::Failed;
            }
            GlobalPlanDraftStatus::Running => {
                match next {
                    GlobalPlanDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanDraftStatus::Done => {
                match next {
                    GlobalPlanDraftStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalPlanDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global plan draft {} cannot move from {} to {}",
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
        format!("{} {}", "global plan draft", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.burst as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPlanDraft {
        GlobalPlanDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            floor_value: self.floor_value,
            cipher: self.cipher.clone(),
            reviewer: self.reviewer.clone(),
            burst: self.burst,
            tenant_id: self.tenant_id.clone(),
            phone: self.phone.clone(),
            expires_at: self.expires_at,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global plan draft in status new.
pub fn new_global_plan_draft(code: &str, title: &str) -> GlobalPlanDraft {
    GlobalPlanDraft {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        floor_value: 0.0,
        cipher: "".to_string(),
        reviewer: "".to_string(),
        burst: 0,
        tenant_id: "".to_string(),
        phone: "".to_string(),
        expires_at: 0,
        window_days: 0,
        status: GlobalPlanDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_plan_draft("code-586", "title-399");
        assert_eq!(item.status, GlobalPlanDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_global_plan_draft("code-586", "title-399");
        if item.move_to(GlobalPlanDraftStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPlanDraftStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_plan_draft("code-586", "title-399");
        item.status = GlobalPlanDraftStatus::Failed;
        assert!(
            item.move_to(GlobalPlanDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
