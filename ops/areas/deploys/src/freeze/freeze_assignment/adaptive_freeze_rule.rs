use super::*;

use acme_kit::text;

/// Where a adaptive freeze rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFreezeRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveFreezeRuleStatus {
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

impl std::fmt::Display for AdaptiveFreezeRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive freeze rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRule {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub ends_at: i64,
    pub port: i64,
    pub sequence: i64,
    pub category: String,
    pub principal: String,
    pub capacity: i64,
    pub status: AdaptiveFreezeRuleStatus,
}

impl AdaptiveFreezeRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFreezeRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFreezeRuleStatus) -> bool {
        match self.status {
            AdaptiveFreezeRuleStatus::Draft => {
                match next {
                    AdaptiveFreezeRuleStatus::Active | AdaptiveFreezeRuleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFreezeRuleStatus::Pending => {
                return next == AdaptiveFreezeRuleStatus::Active;
            }
            AdaptiveFreezeRuleStatus::Active => {
                match next {
                    AdaptiveFreezeRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFreezeRuleStatus::Suspended => {
                match next {
                    AdaptiveFreezeRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveFreezeRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive freeze rule {} cannot move from {} to {}",
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
        format!("{} {}", "adaptive freeze rule", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.port as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFreezeRule {
        AdaptiveFreezeRule {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            port: self.port,
            sequence: self.sequence,
            category: self.category.clone(),
            principal: self.principal.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive freeze rule in status draft.
pub fn new_adaptive_freeze_rule(code: &str, title: &str) -> AdaptiveFreezeRule {
    AdaptiveFreezeRule {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        ends_at: 0,
        port: 0,
        sequence: 0,
        category: "".to_string(),
        principal: "".to_string(),
        capacity: 0,
        status: AdaptiveFreezeRuleStatus::Draft,
    }
}
