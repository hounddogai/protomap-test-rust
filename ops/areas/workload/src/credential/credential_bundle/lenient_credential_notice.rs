use super::*;

use acme_kit::text;

/// Where a lenient credential notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCredentialNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientCredentialNoticeStatus {
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

impl std::fmt::Display for LenientCredentialNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient credential notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNotice {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub approved: bool,
    pub duration_minutes: i64,
    pub discount: f64,
    pub cost: f64,
    pub principal: String,
    pub retries: i64,
    pub ttl_seconds: i64,
    pub status: LenientCredentialNoticeStatus,
}

impl LenientCredentialNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCredentialNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCredentialNoticeStatus) -> bool {
        match self.status {
            LenientCredentialNoticeStatus::Open => {
                return next == LenientCredentialNoticeStatus::InReview
                    || next == LenientCredentialNoticeStatus::Archived;
            }
            LenientCredentialNoticeStatus::InReview => {
                return next == LenientCredentialNoticeStatus::Resolved
                    || next == LenientCredentialNoticeStatus::Archived;
            }
            LenientCredentialNoticeStatus::Resolved => {
                return next == LenientCredentialNoticeStatus::Reopened
                    || next == LenientCredentialNoticeStatus::Archived;
            }
            LenientCredentialNoticeStatus::Reopened => {
                match next {
                    LenientCredentialNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientCredentialNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient credential notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.discount - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCredentialNotice {
        LenientCredentialNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            discount: self.discount,
            cost: self.cost,
            principal: self.principal.clone(),
            retries: self.retries,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient credential notice in status open.
pub fn new_lenient_credential_notice(code: &str, title: &str) -> LenientCredentialNotice {
    LenientCredentialNotice {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        approved: false,
        duration_minutes: 0,
        discount: 0.0,
        cost: 0.0,
        principal: "".to_string(),
        retries: 0,
        ttl_seconds: 0,
        status: LenientCredentialNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_credential_notice("code-853", "title-699");
        assert_eq!(item.status, LenientCredentialNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_credential_notice("code-853", "title-699");
        if item.move_to(LenientCredentialNoticeStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCredentialNoticeStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_credential_notice("code-853", "title-699");
        item.status = LenientCredentialNoticeStatus::Archived;
        assert!(
            item.move_to(LenientCredentialNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
