use super::*;

use acme_kit::text;

/// Where a tiered retention notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRetentionNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredRetentionNoticeStatus {
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

impl std::fmt::Display for TieredRetentionNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered retention notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNotice {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub budget: f64,
    pub threshold: f64,
    pub grace_days: i64,
    pub sequence: i64,
    pub created_at: i64,
    pub name: String,
    pub status: TieredRetentionNoticeStatus,
}

impl TieredRetentionNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRetentionNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRetentionNoticeStatus) -> bool {
        match self.status {
            TieredRetentionNoticeStatus::Open => {
                match next {
                    TieredRetentionNoticeStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRetentionNoticeStatus::InReview => {
                match next {
                    TieredRetentionNoticeStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRetentionNoticeStatus::Resolved => {
                match next {
                    TieredRetentionNoticeStatus::Reopened | TieredRetentionNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRetentionNoticeStatus::Reopened => {
                return next == TieredRetentionNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRetentionNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered retention notice {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "tiered retention notice", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.grace_days as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRetentionNotice {
        TieredRetentionNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            budget: self.budget,
            threshold: self.threshold,
            grace_days: self.grace_days,
            sequence: self.sequence,
            created_at: self.created_at,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered retention notice in status open.
pub fn new_tiered_retention_notice(code: &str, title: &str) -> TieredRetentionNotice {
    TieredRetentionNotice {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        budget: 0.0,
        threshold: 0.0,
        grace_days: 0,
        sequence: 0,
        created_at: 0,
        name: "".to_string(),
        status: TieredRetentionNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_retention_notice("code-161", "title-781");
        assert_eq!(item.status, TieredRetentionNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_retention_notice("code-161", "title-781");
        if item.move_to(TieredRetentionNoticeStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRetentionNoticeStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_retention_notice("code-161", "title-781");
        item.status = TieredRetentionNoticeStatus::Archived;
        assert!(
            item.move_to(TieredRetentionNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
