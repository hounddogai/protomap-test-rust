use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a token digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TokenDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TokenDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenDigest {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub owner_id: String,
    pub reviewer: String,
    pub minimum: f64,
    pub port: i64,
    pub weight: f64,
    pub margin: f64,
    pub discount: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: TokenDigestStatus,
}

impl TokenDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenDigestStatus) -> bool {
        match self.status {
            TokenDigestStatus::Requested => {
                return next == TokenDigestStatus::Revoked;
            }
            TokenDigestStatus::Granted => {
                match next {
                    TokenDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenDigestStatus::Revoked => {
                match next {
                    TokenDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: TokenDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token digest", self.code)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.weight + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenDigest {
        TokenDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            owner_id: self.owner_id.clone(),
            reviewer: self.reviewer.clone(),
            minimum: self.minimum,
            port: self.port,
            weight: self.weight,
            margin: self.margin,
            discount: self.discount,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token digest in status requested.
pub fn new_token_digest(code: &str, name: &str) -> TokenDigest {
    TokenDigest {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        owner_id: "".to_string(),
        reviewer: "".to_string(),
        minimum: 0.0,
        port: 0,
        weight: 0.0,
        margin: 0.0,
        discount: 0.0,
        attributes: BTreeMap::new(),
        status: TokenDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_token_digest("code-250", "name-639");
        assert_eq!(item.status, TokenDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_token_digest("code-250", "name-639");
        if item.move_to(TokenDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_token_digest("code-250", "name-639");
        item.status = TokenDigestStatus::Expired;
        assert!(
            item.move_to(TokenDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
