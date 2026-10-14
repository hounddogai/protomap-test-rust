use super::*;

use acme_kit::text;

/// Where a quote tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QuoteTierStatus {
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

impl std::fmt::Display for QuoteTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote tier as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteTier {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub cipher: String,
    pub floor_value: f64,
    pub locale: String,
    pub ends_at: i64,
    pub score: f64,
    pub archived: bool,
    pub checksum: String,
    pub status: QuoteTierStatus,
}

impl QuoteTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteTierStatus) -> bool {
        match self.status {
            QuoteTierStatus::Open => {
                match next {
                    QuoteTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteTierStatus::InReview => {
                return next == QuoteTierStatus::Resolved || next == QuoteTierStatus::Reopened;
            }
            QuoteTierStatus::Resolved => {
                return next == QuoteTierStatus::Archived;
            }
            QuoteTierStatus::Reopened => {
                match next {
                    QuoteTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: QuoteTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote tier {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteTier {
        QuoteTier {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            cipher: self.cipher.clone(),
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            ends_at: self.ends_at,
            score: self.score,
            archived: self.archived,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a quote tier in status open.
pub fn new_quote_tier(code: &str, title: &str) -> QuoteTier {
    QuoteTier {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        cipher: "".to_string(),
        floor_value: 0.0,
        locale: "".to_string(),
        ends_at: 0,
        score: 0.0,
        archived: false,
        checksum: "".to_string(),
        status: QuoteTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_quote_tier("code-72", "title-822");
        assert_eq!(item.status, QuoteTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_quote_tier("code-72", "title-822");
        if item.move_to(QuoteTierStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteTierStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_quote_tier("code-72", "title-822");
        item.status = QuoteTierStatus::Archived;
        assert!(item.move_to(QuoteTierStatus::Open).is_err(), "expected move to fail");
    }
}
