use super::*;

use acme_kit::text;

/// Where a pending probe draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingProbeDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingProbeDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PendingProbeDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending probe draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeDraft {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub window_days: i64,
    pub owner_id: String,
    pub reviewer: String,
    pub rotation_days: i64,
    pub floor_value: f64,
    pub burst: i64,
    pub active: bool,
    pub status: PendingProbeDraftStatus,
}

impl PendingProbeDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingProbeDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingProbeDraftStatus) -> bool {
        match self.status {
            PendingProbeDraftStatus::Open => {
                match next {
                    PendingProbeDraftStatus::Resolved | PendingProbeDraftStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingProbeDraftStatus::InReview => {
                return next == PendingProbeDraftStatus::Reopened;
            }
            PendingProbeDraftStatus::Resolved => {
                return next == PendingProbeDraftStatus::Reopened || next == PendingProbeDraftStatus::Archived;
            }
            PendingProbeDraftStatus::Reopened => {
                return next == PendingProbeDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingProbeDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending probe draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.window_days as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingProbeDraft {
        PendingProbeDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            owner_id: self.owner_id.clone(),
            reviewer: self.reviewer.clone(),
            rotation_days: self.rotation_days,
            floor_value: self.floor_value,
            burst: self.burst,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending probe draft in status open.
pub fn new_pending_probe_draft(code: &str, label: &str) -> PendingProbeDraft {
    PendingProbeDraft {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        window_days: 0,
        owner_id: "".to_string(),
        reviewer: "".to_string(),
        rotation_days: 0,
        floor_value: 0.0,
        burst: 0,
        active: false,
        status: PendingProbeDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_probe_draft("code-770", "label-729");
        assert_eq!(item.status, PendingProbeDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_pending_probe_draft("code-770", "label-729");
        if item.move_to(PendingProbeDraftStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingProbeDraftStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_probe_draft("code-770", "label-729");
        item.status = PendingProbeDraftStatus::Archived;
        assert!(
            item.move_to(PendingProbeDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
