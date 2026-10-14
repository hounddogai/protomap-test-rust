use super::*;

use acme_kit::text;

/// Where a token plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TokenPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TokenPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenPlan {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub cost: f64,
    pub quota_units: i64,
    pub currency: String,
    pub port: i64,
    pub owner_id: String,
    pub created_at: i64,
    pub issuer: String,
    pub rate: f64,
    pub status: TokenPlanStatus,
}

impl TokenPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenPlanStatus) -> bool {
        match self.status {
            TokenPlanStatus::Created => {
                return next == TokenPlanStatus::Published;
            }
            TokenPlanStatus::Validated => {
                match next {
                    TokenPlanStatus::Published | TokenPlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenPlanStatus::Published => {
                return next == TokenPlanStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token plan {} cannot move from {} to {}",
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
        format!("{} {}", "token plan", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenPlan {
        TokenPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            cost: self.cost,
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            port: self.port,
            owner_id: self.owner_id.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token plan in status created.
pub fn new_token_plan(code: &str, name: &str) -> TokenPlan {
    TokenPlan {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        cost: 0.0,
        quota_units: 0,
        currency: "".to_string(),
        port: 0,
        owner_id: "".to_string(),
        created_at: 0,
        issuer: "".to_string(),
        rate: 0.0,
        status: TokenPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_token_plan("code-101", "name-584");
        assert_eq!(item.status, TokenPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_token_plan("code-101", "name-584");
        if item.move_to(TokenPlanStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenPlanStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_token_plan("code-101", "name-584");
        item.status = TokenPlanStatus::Retired;
        assert!(item.move_to(TokenPlanStatus::Created).is_err(), "expected move to fail");
    }
}
