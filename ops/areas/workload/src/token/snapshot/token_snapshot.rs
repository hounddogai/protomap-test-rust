use super::*;

use acme_kit::text;

/// Where a token snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TokenSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TokenSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSnapshot {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub contact_email: String,
    pub attempts: i64,
    pub status: TokenSnapshotStatus,
}

impl TokenSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenSnapshotStatus) -> bool {
        match self.status {
            TokenSnapshotStatus::Created => {
                match next {
                    TokenSnapshotStatus::Validated | TokenSnapshotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenSnapshotStatus::Validated => {
                match next {
                    TokenSnapshotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenSnapshotStatus::Published => {
                match next {
                    TokenSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: TokenSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.attempts as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenSnapshot {
        TokenSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            contact_email: self.contact_email.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token snapshot in status created.
pub fn new_token_snapshot(code: &str, name: &str) -> TokenSnapshot {
    TokenSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        contact_email: "".to_string(),
        attempts: 0,
        status: TokenSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_token_snapshot("code-649", "name-86");
        assert_eq!(item.status, TokenSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_token_snapshot("code-649", "name-86");
        if item.move_to(TokenSnapshotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenSnapshotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_token_snapshot("code-649", "name-86");
        item.status = TokenSnapshotStatus::Retired;
        assert!(
            item.move_to(TokenSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
