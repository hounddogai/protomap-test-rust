use super::*;

use acme_kit::text;

/// Where a tiered token claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTokenClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredTokenClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredTokenClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered token claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenClaim {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub contact_email: String,
    pub currency: String,
    pub title: String,
    pub status: TieredTokenClaimStatus,
}

impl TieredTokenClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTokenClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTokenClaimStatus) -> bool {
        match self.status {
            TieredTokenClaimStatus::Created => {
                match next {
                    TieredTokenClaimStatus::Validated | TieredTokenClaimStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTokenClaimStatus::Validated => {
                return next == TieredTokenClaimStatus::Published || next == TieredTokenClaimStatus::Retired;
            }
            TieredTokenClaimStatus::Published => {
                return next == TieredTokenClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTokenClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered token claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered token claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> TieredTokenClaim {
        TieredTokenClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered token claim in status created.
pub fn new_tiered_token_claim(code: &str, name: &str) -> TieredTokenClaim {
    TieredTokenClaim {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        contact_email: "".to_string(),
        currency: "".to_string(),
        title: "".to_string(),
        status: TieredTokenClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_token_claim("code-969", "name-257");
        assert_eq!(item.status, TieredTokenClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_token_claim("code-969", "name-257");
        if item.move_to(TieredTokenClaimStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTokenClaimStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_token_claim("code-969", "name-257");
        item.status = TieredTokenClaimStatus::Retired;
        assert!(
            item.move_to(TieredTokenClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
