use super::*;

use acme_kit::text;

/// Where a grant draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantDraftStatus {
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

impl std::fmt::Display for GrantDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraft {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub cost: f64,
    pub starts_at: i64,
    pub minimum: f64,
    pub discount: f64,
    pub version: i64,
    pub cluster: String,
    pub status: GrantDraftStatus,
}

impl GrantDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantDraftStatus) -> bool {
        match self.status {
            GrantDraftStatus::Open => {
                match next {
                    GrantDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantDraftStatus::InReview => {
                return next == GrantDraftStatus::Resolved || next == GrantDraftStatus::Archived;
            }
            GrantDraftStatus::Resolved => {
                match next {
                    GrantDraftStatus::Reopened | GrantDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantDraftStatus::Reopened => {
                match next {
                    GrantDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: GrantDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.minimum + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantDraft {
        GrantDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            cost: self.cost,
            starts_at: self.starts_at,
            minimum: self.minimum,
            discount: self.discount,
            version: self.version,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a grant draft in status open.
pub fn new_grant_draft(code: &str, name: &str) -> GrantDraft {
    GrantDraft {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        cost: 0.0,
        starts_at: 0,
        minimum: 0.0,
        discount: 0.0,
        version: 0,
        cluster: "".to_string(),
        status: GrantDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_draft("code-744", "name-299");
        assert_eq!(item.status, GrantDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_grant_draft("code-744", "name-299");
        if item.move_to(GrantDraftStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantDraftStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_draft("code-744", "name-299");
        item.status = GrantDraftStatus::Archived;
        assert!(item.move_to(GrantDraftStatus::Open).is_err(), "expected move to fail");
    }
}
