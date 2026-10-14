use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a token summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TokenSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TokenSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSummary {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub active: bool,
    pub principal: String,
    pub quota_units: i64,
    pub discount: f64,
    pub port: i64,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub status: TokenSummaryStatus,
}

impl TokenSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenSummaryStatus) -> bool {
        match self.status {
            TokenSummaryStatus::Requested => {
                match next {
                    TokenSummaryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenSummaryStatus::Granted => {
                return next == TokenSummaryStatus::Revoked || next == TokenSummaryStatus::Expired;
            }
            TokenSummaryStatus::Revoked => {
                match next {
                    TokenSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: TokenSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.discount - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenSummary {
        TokenSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            active: self.active,
            principal: self.principal.clone(),
            quota_units: self.quota_units,
            discount: self.discount,
            port: self.port,
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token summary in status requested.
pub fn new_token_summary(code: &str, name: &str) -> TokenSummary {
    TokenSummary {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        active: false,
        principal: "".to_string(),
        quota_units: 0,
        discount: 0.0,
        port: 0,
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        status: TokenSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_token_summary("code-703", "name-6");
        assert_eq!(item.status, TokenSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_token_summary("code-703", "name-6");
        if item.move_to(TokenSummaryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenSummaryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_token_summary("code-703", "name-6");
        item.status = TokenSummaryStatus::Expired;
        assert!(
            item.move_to(TokenSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
