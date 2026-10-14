use super::*;

use acme_kit::text;

/// Where a fallback quote quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackQuoteQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackQuoteQuotaStatus {
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

impl std::fmt::Display for FallbackQuoteQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback quote quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteQuota {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub quantity: i64,
    pub fingerprint: String,
    pub description: String,
    pub sequence: i64,
    pub ratio: f64,
    pub status: FallbackQuoteQuotaStatus,
}

impl FallbackQuoteQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackQuoteQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackQuoteQuotaStatus) -> bool {
        match self.status {
            FallbackQuoteQuotaStatus::Open => {
                match next {
                    FallbackQuoteQuotaStatus::Resolved | FallbackQuoteQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackQuoteQuotaStatus::InReview => {
                return next == FallbackQuoteQuotaStatus::Resolved;
            }
            FallbackQuoteQuotaStatus::Resolved => {
                return next == FallbackQuoteQuotaStatus::Archived;
            }
            FallbackQuoteQuotaStatus::Reopened => {
                return next == FallbackQuoteQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackQuoteQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback quote quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback quote quota", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackQuoteQuota {
        FallbackQuoteQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            quantity: self.quantity,
            fingerprint: self.fingerprint.clone(),
            description: self.description.clone(),
            sequence: self.sequence,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback quote quota in status open.
pub fn new_fallback_quote_quota(code: &str, title: &str) -> FallbackQuoteQuota {
    FallbackQuoteQuota {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        quantity: 0,
        fingerprint: "".to_string(),
        description: "".to_string(),
        sequence: 0,
        ratio: 0.0,
        status: FallbackQuoteQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_quote_quota("code-751", "title-216");
        assert_eq!(item.status, FallbackQuoteQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_fallback_quote_quota("code-751", "title-216");
        if item.move_to(FallbackQuoteQuotaStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackQuoteQuotaStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_quote_quota("code-751", "title-216");
        item.status = FallbackQuoteQuotaStatus::Archived;
        assert!(
            item.move_to(FallbackQuoteQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
