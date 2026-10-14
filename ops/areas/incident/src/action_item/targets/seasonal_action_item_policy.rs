use super::*;

use acme_kit::text;

/// Where a seasonal action item policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalActionItemPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalActionItemPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalActionItemPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal action item policy as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemPolicy {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub external_ref: String,
    pub country: String,
    pub duration_minutes: i64,
    pub phone: String,
    pub tenant_id: String,
    pub status: SeasonalActionItemPolicyStatus,
}

impl SeasonalActionItemPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalActionItemPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalActionItemPolicyStatus) -> bool {
        match self.status {
            SeasonalActionItemPolicyStatus::Requested => {
                match next {
                    SeasonalActionItemPolicyStatus::Granted | SeasonalActionItemPolicyStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalActionItemPolicyStatus::Granted => {
                return next == SeasonalActionItemPolicyStatus::Revoked;
            }
            SeasonalActionItemPolicyStatus::Revoked => {
                match next {
                    SeasonalActionItemPolicyStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalActionItemPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal action item policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalActionItemPolicy {
        SeasonalActionItemPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal action item policy in status requested.
pub fn new_seasonal_action_item_policy(code: &str, name: &str) -> SeasonalActionItemPolicy {
    SeasonalActionItemPolicy {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        external_ref: "".to_string(),
        country: "".to_string(),
        duration_minutes: 0,
        phone: "".to_string(),
        tenant_id: "".to_string(),
        status: SeasonalActionItemPolicyStatus::Requested,
    }
}
