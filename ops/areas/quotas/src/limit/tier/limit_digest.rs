use super::*;

use acme_kit::text;

/// Where a limit digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LimitDigestStatus {
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

impl std::fmt::Display for LimitDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigest {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub revision: i64,
    pub rotation_days: i64,
    pub region: String,
    pub created_at: i64,
    pub status: LimitDigestStatus,
}

impl LimitDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitDigestStatus) -> bool {
        match self.status {
            LimitDigestStatus::Open => {
                match next {
                    LimitDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitDigestStatus::InReview => {
                return next == LimitDigestStatus::Archived;
            }
            LimitDigestStatus::Resolved => {
                return next == LimitDigestStatus::Reopened;
            }
            LimitDigestStatus::Reopened => {
                match next {
                    LimitDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: LimitDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.revision as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitDigest {
        LimitDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            revision: self.revision,
            rotation_days: self.rotation_days,
            region: self.region.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a limit digest in status open.
pub fn new_limit_digest(code: &str, label: &str) -> LimitDigest {
    LimitDigest {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        revision: 0,
        rotation_days: 0,
        region: "".to_string(),
        created_at: 0,
        status: LimitDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_limit_digest("code-907", "label-661");
        assert_eq!(item.status, LimitDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_limit_digest("code-907", "label-661");
        if item.move_to(LimitDigestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitDigestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_limit_digest("code-907", "label-661");
        item.status = LimitDigestStatus::Archived;
        assert!(item.move_to(LimitDigestStatus::Open).is_err(), "expected move to fail");
    }
}
