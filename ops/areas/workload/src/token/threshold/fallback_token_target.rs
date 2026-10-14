use super::*;

use acme_kit::text;

/// Where a fallback token target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTokenTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackTokenTargetStatus {
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

impl std::fmt::Display for FallbackTokenTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback token target as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTarget {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub limit: i64,
    pub quantity: i64,
    pub burst: i64,
    pub fingerprint: String,
    pub grace_days: i64,
    pub ttl_seconds: i64,
    pub phone: String,
    pub title: String,
    pub status: FallbackTokenTargetStatus,
}

impl FallbackTokenTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTokenTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTokenTargetStatus) -> bool {
        match self.status {
            FallbackTokenTargetStatus::Open => {
                match next {
                    FallbackTokenTargetStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTokenTargetStatus::InReview => {
                match next {
                    FallbackTokenTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTokenTargetStatus::Resolved => {
                return next == FallbackTokenTargetStatus::Reopened || next == FallbackTokenTargetStatus::Archived;
            }
            FallbackTokenTargetStatus::Reopened => {
                return next == FallbackTokenTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackTokenTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback token target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.burst as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTokenTarget {
        FallbackTokenTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            limit: self.limit,
            quantity: self.quantity,
            burst: self.burst,
            fingerprint: self.fingerprint.clone(),
            grace_days: self.grace_days,
            ttl_seconds: self.ttl_seconds,
            phone: self.phone.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback token target in status open.
pub fn new_fallback_token_target(code: &str, name: &str) -> FallbackTokenTarget {
    FallbackTokenTarget {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        limit: 0,
        quantity: 0,
        burst: 0,
        fingerprint: "".to_string(),
        grace_days: 0,
        ttl_seconds: 0,
        phone: "".to_string(),
        title: "".to_string(),
        status: FallbackTokenTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_token_target("code-653", "name-927");
        assert_eq!(item.status, FallbackTokenTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_token_target("code-653", "name-927");
        if item.move_to(FallbackTokenTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTokenTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_token_target("code-653", "name-927");
        item.status = FallbackTokenTargetStatus::Archived;
        assert!(
            item.move_to(FallbackTokenTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
