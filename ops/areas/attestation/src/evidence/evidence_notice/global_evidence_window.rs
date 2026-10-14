use super::*;

use acme_kit::text;

/// Where a global evidence window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEvidenceWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalEvidenceWindowStatus {
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

impl std::fmt::Display for GlobalEvidenceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global evidence window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindow {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub updated_at: i64,
    pub margin: f64,
    pub status: GlobalEvidenceWindowStatus,
}

impl GlobalEvidenceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEvidenceWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEvidenceWindowStatus) -> bool {
        match self.status {
            GlobalEvidenceWindowStatus::New => {
                return next == GlobalEvidenceWindowStatus::Done;
            }
            GlobalEvidenceWindowStatus::Queued => {
                match next {
                    GlobalEvidenceWindowStatus::Done | GlobalEvidenceWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEvidenceWindowStatus::Running => {
                return next == GlobalEvidenceWindowStatus::Done || next == GlobalEvidenceWindowStatus::Failed;
            }
            GlobalEvidenceWindowStatus::Done => {
                return next == GlobalEvidenceWindowStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalEvidenceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global evidence window {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.updated_at as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEvidenceWindow {
        GlobalEvidenceWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            updated_at: self.updated_at,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global evidence window in status new.
pub fn new_global_evidence_window(code: &str, title: &str) -> GlobalEvidenceWindow {
    GlobalEvidenceWindow {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        updated_at: 0,
        margin: 0.0,
        status: GlobalEvidenceWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_evidence_window("code-443", "title-583");
        assert_eq!(item.status, GlobalEvidenceWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_global_evidence_window("code-443", "title-583");
        if item.move_to(GlobalEvidenceWindowStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalEvidenceWindowStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_evidence_window("code-443", "title-583");
        item.status = GlobalEvidenceWindowStatus::Failed;
        assert!(
            item.move_to(GlobalEvidenceWindowStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
