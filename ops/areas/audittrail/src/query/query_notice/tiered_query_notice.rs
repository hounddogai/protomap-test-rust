use super::*;

use acme_kit::text;

/// Where a tiered query notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredQueryNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredQueryNoticeStatus {
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

impl std::fmt::Display for TieredQueryNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered query notice as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryNotice {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub unit: String,
    pub sequence: i64,
    pub description: String,
    pub minimum: f64,
    pub grace_days: i64,
    pub ends_at: i64,
    pub status: TieredQueryNoticeStatus,
}

impl TieredQueryNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredQueryNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredQueryNoticeStatus) -> bool {
        match self.status {
            TieredQueryNoticeStatus::Open => {
                return next == TieredQueryNoticeStatus::Reopened;
            }
            TieredQueryNoticeStatus::InReview => {
                match next {
                    TieredQueryNoticeStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredQueryNoticeStatus::Resolved => {
                return next == TieredQueryNoticeStatus::Reopened || next == TieredQueryNoticeStatus::Archived;
            }
            TieredQueryNoticeStatus::Reopened => {
                return next == TieredQueryNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredQueryNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered query notice {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "tiered query notice", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ends_at as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredQueryNotice {
        TieredQueryNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            unit: self.unit.clone(),
            sequence: self.sequence,
            description: self.description.clone(),
            minimum: self.minimum,
            grace_days: self.grace_days,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered query notice in status open.
pub fn new_tiered_query_notice(code: &str, title: &str) -> TieredQueryNotice {
    TieredQueryNotice {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        unit: "".to_string(),
        sequence: 0,
        description: "".to_string(),
        minimum: 0.0,
        grace_days: 0,
        ends_at: 0,
        status: TieredQueryNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_query_notice("code-892", "title-326");
        assert_eq!(item.status, TieredQueryNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_query_notice("code-892", "title-326");
        if item.move_to(TieredQueryNoticeStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredQueryNoticeStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_query_notice("code-892", "title-326");
        item.status = TieredQueryNoticeStatus::Archived;
        assert!(
            item.move_to(TieredQueryNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
