use super::*;

use acme_kit::text;

/// Where a pending query bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQueryBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingQueryBundleStatus {
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

impl std::fmt::Display for PendingQueryBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending query bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryBundle {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub discount: f64,
    pub retries: i64,
    pub audience: String,
    pub burst: i64,
    pub status: PendingQueryBundleStatus,
}

impl PendingQueryBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQueryBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQueryBundleStatus) -> bool {
        match self.status {
            PendingQueryBundleStatus::Open => {
                return next == PendingQueryBundleStatus::Reopened || next == PendingQueryBundleStatus::Archived;
            }
            PendingQueryBundleStatus::InReview => {
                match next {
                    PendingQueryBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingQueryBundleStatus::Resolved => {
                match next {
                    PendingQueryBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingQueryBundleStatus::Reopened => {
                match next {
                    PendingQueryBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingQueryBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending query bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.burst as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingQueryBundle {
        PendingQueryBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            discount: self.discount,
            retries: self.retries,
            audience: self.audience.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending query bundle in status open.
pub fn new_pending_query_bundle(code: &str, title: &str) -> PendingQueryBundle {
    PendingQueryBundle {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        discount: 0.0,
        retries: 0,
        audience: "".to_string(),
        burst: 0,
        status: PendingQueryBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_query_bundle("code-563", "title-987");
        assert_eq!(item.status, PendingQueryBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_pending_query_bundle("code-563", "title-987");
        if item.move_to(PendingQueryBundleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingQueryBundleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_query_bundle("code-563", "title-987");
        item.status = PendingQueryBundleStatus::Archived;
        assert!(
            item.move_to(PendingQueryBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
