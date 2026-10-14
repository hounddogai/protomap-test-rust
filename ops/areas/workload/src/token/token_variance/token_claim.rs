use super::*;

use acme_kit::text;

/// Where a token claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TokenClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TokenClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenClaim {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub created_at: i64,
    pub active: bool,
    pub expires_at: i64,
    pub approved: bool,
    pub currency: String,
    pub status: TokenClaimStatus,
}

impl TokenClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenClaimStatus) -> bool {
        match self.status {
            TokenClaimStatus::Planned => {
                return next == TokenClaimStatus::Completed || next == TokenClaimStatus::Cancelled;
            }
            TokenClaimStatus::Scheduled => {
                match next {
                    TokenClaimStatus::InProgress | TokenClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenClaimStatus::InProgress => {
                return next == TokenClaimStatus::Cancelled;
            }
            TokenClaimStatus::Completed => {
                match next {
                    TokenClaimStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TokenClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token claim {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.created_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenClaim {
        TokenClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            created_at: self.created_at,
            active: self.active,
            expires_at: self.expires_at,
            approved: self.approved,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a token claim in status planned.
pub fn new_token_claim(code: &str, name: &str) -> TokenClaim {
    TokenClaim {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        created_at: 0,
        active: false,
        expires_at: 0,
        approved: false,
        currency: "".to_string(),
        status: TokenClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_token_claim("code-746", "name-185");
        assert_eq!(item.status, TokenClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_token_claim("code-746", "name-185");
        if item.move_to(TokenClaimStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenClaimStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_token_claim("code-746", "name-185");
        item.status = TokenClaimStatus::Cancelled;
        assert!(
            item.move_to(TokenClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
