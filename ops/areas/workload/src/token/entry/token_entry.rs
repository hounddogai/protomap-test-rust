use super::*;

use acme_kit::text;

/// Where a token entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TokenEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TokenEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntry {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub attempts: i64,
    pub owner_id: String,
    pub ceiling: f64,
    pub key_id: String,
    pub limit: i64,
    pub namespace: String,
    pub status: TokenEntryStatus,
}

impl TokenEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenEntryStatus) -> bool {
        match self.status {
            TokenEntryStatus::Created => {
                return next == TokenEntryStatus::Validated || next == TokenEntryStatus::Published;
            }
            TokenEntryStatus::Validated => {
                match next {
                    TokenEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenEntryStatus::Published => {
                return next == TokenEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token entry", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ceiling + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenEntry {
        TokenEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            owner_id: self.owner_id.clone(),
            ceiling: self.ceiling,
            key_id: self.key_id.clone(),
            limit: self.limit,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a token entry in status created.
pub fn new_token_entry(code: &str, name: &str) -> TokenEntry {
    TokenEntry {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        attempts: 0,
        owner_id: "".to_string(),
        ceiling: 0.0,
        key_id: "".to_string(),
        limit: 0,
        namespace: "".to_string(),
        status: TokenEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_token_entry("code-942", "name-721");
        assert_eq!(item.status, TokenEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_token_entry("code-942", "name-721");
        if item.move_to(TokenEntryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenEntryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_token_entry("code-942", "name-721");
        item.status = TokenEntryStatus::Retired;
        assert!(
            item.move_to(TokenEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
