use super::*;

use acme_kit::text;

/// Where a ttl policy tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TtlPolicyTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TtlPolicyTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy tier as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTier {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub maximum: f64,
    pub grace_days: i64,
    pub segment: String,
    pub status: TtlPolicyTierStatus,
}

impl TtlPolicyTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyTierStatus) -> bool {
        match self.status {
            TtlPolicyTierStatus::Created => {
                match next {
                    TtlPolicyTierStatus::Validated | TtlPolicyTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyTierStatus::Validated => {
                return next == TtlPolicyTierStatus::Published;
            }
            TtlPolicyTierStatus::Published => {
                return next == TtlPolicyTierStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.maximum - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyTier {
        TtlPolicyTier {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            maximum: self.maximum,
            grace_days: self.grace_days,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a ttl policy tier in status created.
pub fn new_ttl_policy_tier(code: &str, name: &str) -> TtlPolicyTier {
    TtlPolicyTier {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        maximum: 0.0,
        grace_days: 0,
        segment: "".to_string(),
        status: TtlPolicyTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_ttl_policy_tier("code-578", "name-870");
        assert_eq!(item.status, TtlPolicyTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_ttl_policy_tier("code-578", "name-870");
        if item.move_to(TtlPolicyTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_ttl_policy_tier("code-578", "name-870");
        item.status = TtlPolicyTierStatus::Retired;
        assert!(
            item.move_to(TtlPolicyTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
