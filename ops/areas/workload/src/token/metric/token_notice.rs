use super::*;

use acme_kit::text;

/// Where a token notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TokenNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TokenNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenNotice {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub cost: f64,
    pub budget: f64,
    pub ends_at: i64,
    pub unit: String,
    pub source: String,
    pub status: TokenNoticeStatus,
}

impl TokenNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenNoticeStatus) -> bool {
        match self.status {
            TokenNoticeStatus::Proposed => {
                return next == TokenNoticeStatus::Approved || next == TokenNoticeStatus::Rejected;
            }
            TokenNoticeStatus::Approved => {
                match next {
                    TokenNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenNoticeStatus::Rejected => {
                return next == TokenNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token notice", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenNotice {
        TokenNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            cost: self.cost,
            budget: self.budget,
            ends_at: self.ends_at,
            unit: self.unit.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a token notice in status proposed.
pub fn new_token_notice(code: &str, title: &str) -> TokenNotice {
    TokenNotice {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        cost: 0.0,
        budget: 0.0,
        ends_at: 0,
        unit: "".to_string(),
        source: "".to_string(),
        status: TokenNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_token_notice("code-817", "title-963");
        assert_eq!(item.status, TokenNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_token_notice("code-817", "title-963");
        if item.move_to(TokenNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_token_notice("code-817", "title-963");
        item.status = TokenNoticeStatus::Withdrawn;
        assert!(
            item.move_to(TokenNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
