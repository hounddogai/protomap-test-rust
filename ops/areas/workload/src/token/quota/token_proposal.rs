use super::*;

use acme_kit::text;

/// Where a token proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TokenProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TokenProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenProposal {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub retries: i64,
    pub issuer: String,
    pub grace_days: i64,
    pub quantity: i64,
    pub title: String,
    pub floor_value: f64,
    pub category: String,
    pub status: TokenProposalStatus,
}

impl TokenProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenProposalStatus) -> bool {
        match self.status {
            TokenProposalStatus::Created => {
                match next {
                    TokenProposalStatus::Validated | TokenProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenProposalStatus::Validated => {
                match next {
                    TokenProposalStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenProposalStatus::Published => {
                match next {
                    TokenProposalStatus::Retired => {
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

    pub fn move_to(&mut self, next: TokenProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token proposal", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenProposal {
        TokenProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            retries: self.retries,
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            quantity: self.quantity,
            title: self.title.clone(),
            floor_value: self.floor_value,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a token proposal in status created.
pub fn new_token_proposal(code: &str, name: &str) -> TokenProposal {
    TokenProposal {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        retries: 0,
        issuer: "".to_string(),
        grace_days: 0,
        quantity: 0,
        title: "".to_string(),
        floor_value: 0.0,
        category: "".to_string(),
        status: TokenProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_token_proposal("code-660", "name-797");
        assert_eq!(item.status, TokenProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_token_proposal("code-660", "name-797");
        if item.move_to(TokenProposalStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenProposalStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_token_proposal("code-660", "name-797");
        item.status = TokenProposalStatus::Retired;
        assert!(
            item.move_to(TokenProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
