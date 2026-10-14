use super::*;

use acme_kit::text;

/// Where a grant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantTargetStatus {
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

impl std::fmt::Display for GrantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant target as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantTarget {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub revision: i64,
    pub summary: String,
    pub external_ref: String,
    pub starts_at: i64,
    pub priority: i64,
    pub status: GrantTargetStatus,
}

impl GrantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantTargetStatus) -> bool {
        match self.status {
            GrantTargetStatus::Open => {
                return next == GrantTargetStatus::Resolved;
            }
            GrantTargetStatus::InReview => {
                match next {
                    GrantTargetStatus::Reopened | GrantTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantTargetStatus::Resolved => {
                return next == GrantTargetStatus::Reopened || next == GrantTargetStatus::Archived;
            }
            GrantTargetStatus::Reopened => {
                return next == GrantTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.minimum * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantTarget {
        GrantTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            revision: self.revision,
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a grant target in status open.
pub fn new_grant_target(code: &str, title: &str) -> GrantTarget {
    GrantTarget {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        revision: 0,
        summary: "".to_string(),
        external_ref: "".to_string(),
        starts_at: 0,
        priority: 0,
        status: GrantTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_target("code-402", "title-231");
        assert_eq!(item.status, GrantTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_grant_target("code-402", "title-231");
        if item.move_to(GrantTargetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantTargetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_target("code-402", "title-231");
        item.status = GrantTargetStatus::Archived;
        assert!(item.move_to(GrantTargetStatus::Open).is_err(), "expected move to fail");
    }
}
