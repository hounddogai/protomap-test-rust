use super::*;

use acme_kit::text;

/// Where a seasonal isolation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIsolationRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalIsolationRuleStatus {
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

impl std::fmt::Display for SeasonalIsolationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal isolation rule as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationRule {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub updated_at: i64,
    pub attempts: i64,
    pub phone: String,
    pub cipher: String,
    pub quantity: i64,
    pub weight: f64,
    pub window_days: i64,
    pub status: SeasonalIsolationRuleStatus,
}

impl SeasonalIsolationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIsolationRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIsolationRuleStatus) -> bool {
        match self.status {
            SeasonalIsolationRuleStatus::Draft => {
                match next {
                    SeasonalIsolationRuleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIsolationRuleStatus::Pending => {
                return next == SeasonalIsolationRuleStatus::Suspended || next == SeasonalIsolationRuleStatus::Closed;
            }
            SeasonalIsolationRuleStatus::Active => {
                return next == SeasonalIsolationRuleStatus::Closed;
            }
            SeasonalIsolationRuleStatus::Suspended => {
                return next == SeasonalIsolationRuleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalIsolationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal isolation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal isolation rule", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.window_days as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalIsolationRule {
        SeasonalIsolationRule {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            updated_at: self.updated_at,
            attempts: self.attempts,
            phone: self.phone.clone(),
            cipher: self.cipher.clone(),
            quantity: self.quantity,
            weight: self.weight,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal isolation rule in status draft.
pub fn new_seasonal_isolation_rule(code: &str, name: &str) -> SeasonalIsolationRule {
    SeasonalIsolationRule {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        updated_at: 0,
        attempts: 0,
        phone: "".to_string(),
        cipher: "".to_string(),
        quantity: 0,
        weight: 0.0,
        window_days: 0,
        status: SeasonalIsolationRuleStatus::Draft,
    }
}
