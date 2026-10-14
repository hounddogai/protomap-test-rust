use super::*;

use acme_kit::text;

/// Where a token target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TokenTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TokenTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenTarget {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub channel: String,
    pub created_at: i64,
    pub minimum: f64,
    pub locale: String,
    pub floor_value: f64,
    pub status: TokenTargetStatus,
}

impl TokenTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenTargetStatus) -> bool {
        match self.status {
            TokenTargetStatus::Created => {
                match next {
                    TokenTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenTargetStatus::Validated => {
                match next {
                    TokenTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenTargetStatus::Published => {
                match next {
                    TokenTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: TokenTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token target {} cannot move from {} to {}",
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
        format!("{} {}", "token target", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.floor_value - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenTarget {
        TokenTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            channel: self.channel.clone(),
            created_at: self.created_at,
            minimum: self.minimum,
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a token target in status created.
pub fn new_token_target(code: &str, name: &str) -> TokenTarget {
    TokenTarget {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        channel: "".to_string(),
        created_at: 0,
        minimum: 0.0,
        locale: "".to_string(),
        floor_value: 0.0,
        status: TokenTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_token_target("code-814", "name-721");
        assert_eq!(item.status, TokenTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_token_target("code-814", "name-721");
        if item.move_to(TokenTargetStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenTargetStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_token_target("code-814", "name-721");
        item.status = TokenTargetStatus::Retired;
        assert!(
            item.move_to(TokenTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
