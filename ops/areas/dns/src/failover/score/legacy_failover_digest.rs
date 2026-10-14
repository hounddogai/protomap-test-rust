use super::*;

use acme_kit::text;

/// Where a legacy failover digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFailoverDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyFailoverDigestStatus {
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

impl std::fmt::Display for LegacyFailoverDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy failover digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFailoverDigest {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub expires_at: i64,
    pub segment: String,
    pub source: String,
    pub cost: f64,
    pub status: LegacyFailoverDigestStatus,
}

impl LegacyFailoverDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFailoverDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFailoverDigestStatus) -> bool {
        match self.status {
            LegacyFailoverDigestStatus::Open => {
                match next {
                    LegacyFailoverDigestStatus::InReview | LegacyFailoverDigestStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFailoverDigestStatus::InReview => {
                match next {
                    LegacyFailoverDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFailoverDigestStatus::Resolved => {
                return next == LegacyFailoverDigestStatus::Archived;
            }
            LegacyFailoverDigestStatus::Reopened => {
                match next {
                    LegacyFailoverDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyFailoverDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy failover digest {} cannot move from {} to {}",
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
        format!("{} {}", "legacy failover digest", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ends_at as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFailoverDigest {
        LegacyFailoverDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            expires_at: self.expires_at,
            segment: self.segment.clone(),
            source: self.source.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy failover digest in status open.
pub fn new_legacy_failover_digest(code: &str, title: &str) -> LegacyFailoverDigest {
    LegacyFailoverDigest {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        expires_at: 0,
        segment: "".to_string(),
        source: "".to_string(),
        cost: 0.0,
        status: LegacyFailoverDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_failover_digest("code-235", "title-786");
        assert_eq!(item.status, LegacyFailoverDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_legacy_failover_digest("code-235", "title-786");
        if item.move_to(LegacyFailoverDigestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFailoverDigestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_failover_digest("code-235", "title-786");
        item.status = LegacyFailoverDigestStatus::Archived;
        assert!(
            item.move_to(LegacyFailoverDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
