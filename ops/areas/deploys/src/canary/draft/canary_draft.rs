use super::*;

use acme_kit::text;

/// Where a canary draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryDraftStatus {
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

impl std::fmt::Display for CanaryDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraft {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub starts_at: i64,
    pub unit: String,
    pub pinned: bool,
    pub status: CanaryDraftStatus,
}

impl CanaryDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryDraftStatus) -> bool {
        match self.status {
            CanaryDraftStatus::Open => {
                return next == CanaryDraftStatus::InReview || next == CanaryDraftStatus::Archived;
            }
            CanaryDraftStatus::InReview => {
                return next == CanaryDraftStatus::Reopened || next == CanaryDraftStatus::Archived;
            }
            CanaryDraftStatus::Resolved => {
                match next {
                    CanaryDraftStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryDraftStatus::Reopened => {
                match next {
                    CanaryDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: CanaryDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.limit as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryDraft {
        CanaryDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            starts_at: self.starts_at,
            unit: self.unit.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a canary draft in status open.
pub fn new_canary_draft(code: &str, name: &str) -> CanaryDraft {
    CanaryDraft {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        starts_at: 0,
        unit: "".to_string(),
        pinned: false,
        status: CanaryDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_canary_draft("code-438", "name-573");
        assert_eq!(item.status, CanaryDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_canary_draft("code-438", "name-573");
        if item.move_to(CanaryDraftStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryDraftStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_canary_draft("code-438", "name-573");
        item.status = CanaryDraftStatus::Archived;
        assert!(item.move_to(CanaryDraftStatus::Open).is_err(), "expected move to fail");
    }
}
