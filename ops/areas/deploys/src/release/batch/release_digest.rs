use super::*;

use acme_kit::text;

/// Where a release digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseDigestStatus {
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

impl std::fmt::Display for ReleaseDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release digest as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigest {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub name: String,
    pub checksum: String,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub status: ReleaseDigestStatus,
}

impl ReleaseDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseDigestStatus) -> bool {
        match self.status {
            ReleaseDigestStatus::Open => {
                match next {
                    ReleaseDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseDigestStatus::InReview => {
                match next {
                    ReleaseDigestStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseDigestStatus::Resolved => {
                return next == ReleaseDigestStatus::Reopened || next == ReleaseDigestStatus::Archived;
            }
            ReleaseDigestStatus::Reopened => {
                match next {
                    ReleaseDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReleaseDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.cost * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseDigest {
        ReleaseDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a release digest in status open.
pub fn new_release_digest(code: &str, label: &str) -> ReleaseDigest {
    ReleaseDigest {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        name: "".to_string(),
        checksum: "".to_string(),
        archived: false,
        ttl_seconds: 0,
        margin: 0.0,
        status: ReleaseDigestStatus::Open,
    }
}
