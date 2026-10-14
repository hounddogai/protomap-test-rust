use super::*;

use acme_kit::text;

/// Where a tiered credential rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCredentialRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredCredentialRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TieredCredentialRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered credential rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRule {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub expires_at: i64,
    pub quantity: i64,
    pub status: TieredCredentialRuleStatus,
}

impl TieredCredentialRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCredentialRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCredentialRuleStatus) -> bool {
        match self.status {
            TieredCredentialRuleStatus::Draft => {
                return next == TieredCredentialRuleStatus::Pending;
            }
            TieredCredentialRuleStatus::Pending => {
                return next == TieredCredentialRuleStatus::Suspended || next == TieredCredentialRuleStatus::Closed;
            }
            TieredCredentialRuleStatus::Active => {
                match next {
                    TieredCredentialRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCredentialRuleStatus::Suspended => {
                match next {
                    TieredCredentialRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredCredentialRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered credential rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.expires_at as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCredentialRule {
        TieredCredentialRule {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            expires_at: self.expires_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered credential rule in status draft.
pub fn new_tiered_credential_rule(code: &str, title: &str) -> TieredCredentialRule {
    TieredCredentialRule {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        expires_at: 0,
        quantity: 0,
        status: TieredCredentialRuleStatus::Draft,
    }
}
