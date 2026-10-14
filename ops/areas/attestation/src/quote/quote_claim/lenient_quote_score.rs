use super::*;

use acme_kit::text;

/// Where a lenient quote score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQuoteScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientQuoteScoreStatus {
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

impl std::fmt::Display for LenientQuoteScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient quote score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteScore {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub ttl_seconds: i64,
    pub batch_size: i64,
    pub issuer: String,
    pub capacity: i64,
    pub minimum: f64,
    pub status: LenientQuoteScoreStatus,
}

impl LenientQuoteScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQuoteScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQuoteScoreStatus) -> bool {
        match self.status {
            LenientQuoteScoreStatus::Open => {
                return next == LenientQuoteScoreStatus::Reopened || next == LenientQuoteScoreStatus::Archived;
            }
            LenientQuoteScoreStatus::InReview => {
                match next {
                    LenientQuoteScoreStatus::Reopened | LenientQuoteScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuoteScoreStatus::Resolved => {
                return next == LenientQuoteScoreStatus::Reopened;
            }
            LenientQuoteScoreStatus::Reopened => {
                match next {
                    LenientQuoteScoreStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientQuoteScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient quote score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient quote score", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.minimum + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientQuoteScore {
        LenientQuoteScore {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            capacity: self.capacity,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient quote score in status open.
pub fn new_lenient_quote_score(code: &str, name: &str) -> LenientQuoteScore {
    LenientQuoteScore {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        ttl_seconds: 0,
        batch_size: 0,
        issuer: "".to_string(),
        capacity: 0,
        minimum: 0.0,
        status: LenientQuoteScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_quote_score("code-273", "name-876");
        assert_eq!(item.status, LenientQuoteScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_lenient_quote_score("code-273", "name-876");
        if item.move_to(LenientQuoteScoreStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientQuoteScoreStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_quote_score("code-273", "name-876");
        item.status = LenientQuoteScoreStatus::Archived;
        assert!(
            item.move_to(LenientQuoteScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
