use super::*;

use acme_kit::text;

/// Where a token record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TokenRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TokenRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenRecord {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub updated_at: i64,
    pub floor_value: f64,
    pub score: f64,
    pub active: bool,
    pub attempts: i64,
    pub namespace: String,
    pub description: String,
    pub status: TokenRecordStatus,
}

impl TokenRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenRecordStatus) -> bool {
        match self.status {
            TokenRecordStatus::Requested => {
                match next {
                    TokenRecordStatus::Granted | TokenRecordStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenRecordStatus::Granted => {
                match next {
                    TokenRecordStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenRecordStatus::Revoked => {
                return next == TokenRecordStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.attempts as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenRecord {
        TokenRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            updated_at: self.updated_at,
            floor_value: self.floor_value,
            score: self.score,
            active: self.active,
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a token record in status requested.
pub fn new_token_record(code: &str, name: &str) -> TokenRecord {
    TokenRecord {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        updated_at: 0,
        floor_value: 0.0,
        score: 0.0,
        active: false,
        attempts: 0,
        namespace: "".to_string(),
        description: "".to_string(),
        status: TokenRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_token_record("code-407", "name-600");
        assert_eq!(item.status, TokenRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_token_record("code-407", "name-600");
        if item.move_to(TokenRecordStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenRecordStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_token_record("code-407", "name-600");
        item.status = TokenRecordStatus::Expired;
        assert!(
            item.move_to(TokenRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
