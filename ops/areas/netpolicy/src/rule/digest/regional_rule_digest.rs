use super::*;

use acme_kit::text;

/// Where a regional rule digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRuleDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalRuleDigestStatus {
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

impl std::fmt::Display for RegionalRuleDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional rule digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleDigest {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub priority: i64,
    pub discount: f64,
    pub ttl_seconds: i64,
    pub description: String,
    pub reviewer: String,
    pub ceiling: f64,
    pub weight: f64,
    pub maximum: f64,
    pub status: RegionalRuleDigestStatus,
}

impl RegionalRuleDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRuleDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRuleDigestStatus) -> bool {
        match self.status {
            RegionalRuleDigestStatus::Draft => {
                match next {
                    RegionalRuleDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRuleDigestStatus::Pending => {
                match next {
                    RegionalRuleDigestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRuleDigestStatus::Active => {
                return next == RegionalRuleDigestStatus::Suspended;
            }
            RegionalRuleDigestStatus::Suspended => {
                return next == RegionalRuleDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRuleDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rule digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional rule digest", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.weight * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRuleDigest {
        RegionalRuleDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            priority: self.priority,
            discount: self.discount,
            ttl_seconds: self.ttl_seconds,
            description: self.description.clone(),
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            weight: self.weight,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional rule digest in status draft.
pub fn new_regional_rule_digest(code: &str, name: &str) -> RegionalRuleDigest {
    RegionalRuleDigest {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        priority: 0,
        discount: 0.0,
        ttl_seconds: 0,
        description: "".to_string(),
        reviewer: "".to_string(),
        ceiling: 0.0,
        weight: 0.0,
        maximum: 0.0,
        status: RegionalRuleDigestStatus::Draft,
    }
}
