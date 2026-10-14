use super::*;

use acme_kit::text;

/// Where a token quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TokenQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TokenQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuota {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub key_id: String,
    pub external_ref: String,
    pub ceiling: f64,
    pub floor_value: f64,
    pub budget: f64,
    pub pinned: bool,
    pub capacity: i64,
    pub status: TokenQuotaStatus,
}

impl TokenQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenQuotaStatus) -> bool {
        match self.status {
            TokenQuotaStatus::Requested => {
                return next == TokenQuotaStatus::Expired;
            }
            TokenQuotaStatus::Granted => {
                return next == TokenQuotaStatus::Revoked;
            }
            TokenQuotaStatus::Revoked => {
                match next {
                    TokenQuotaStatus::Expired => {
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

    pub fn move_to(&mut self, next: TokenQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.floor_value + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenQuota {
        TokenQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            budget: self.budget,
            pinned: self.pinned,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token quota in status requested.
pub fn new_token_quota(code: &str, name: &str) -> TokenQuota {
    TokenQuota {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        key_id: "".to_string(),
        external_ref: "".to_string(),
        ceiling: 0.0,
        floor_value: 0.0,
        budget: 0.0,
        pinned: false,
        capacity: 0,
        status: TokenQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_token_quota("code-286", "name-336");
        assert_eq!(item.status, TokenQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_token_quota("code-286", "name-336");
        if item.move_to(TokenQuotaStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenQuotaStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_token_quota("code-286", "name-336");
        item.status = TokenQuotaStatus::Expired;
        assert!(
            item.move_to(TokenQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
