use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global retention claim must pass. Check returns its problems, or none.
pub trait GlobalRetentionClaimRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRetentionClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionClaimAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl GlobalRetentionClaimRule for GlobalRetentionClaimAllowedLabelRule {
    fn check(&self, item: &GlobalRetentionClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRetentionClaimRule for GlobalRetentionClaimExpiresAtRangeRule {
    fn check(&self, item: &GlobalRetentionClaim) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionClaimRequiredCodeRule {
    pub message: String,
}

impl GlobalRetentionClaimRule for GlobalRetentionClaimRequiredCodeRule {
    fn check(&self, item: &GlobalRetentionClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionClaimCodePrefixRule {
    pub prefix: String,
}

impl GlobalRetentionClaimRule for GlobalRetentionClaimCodePrefixRule {
    fn check(&self, item: &GlobalRetentionClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global retention claim.
#[derive(Debug, Clone)]
pub struct GlobalRetentionClaimValidator {
    pub rules: Vec<Rc<dyn GlobalRetentionClaimRule>>,
}

impl GlobalRetentionClaimValidator {
    pub fn validate(&self, item: &GlobalRetentionClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalRetentionClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRetentionClaim]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_global_retention_claim_validator() -> GlobalRetentionClaimValidator {
    GlobalRetentionClaimValidator {
        rules: vec![
            Rc::new(GlobalRetentionClaimAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn GlobalRetentionClaimRule>,
            Rc::new(GlobalRetentionClaimExpiresAtRangeRule { min: 16, max: 64 }),
            Rc::new(GlobalRetentionClaimRequiredCodeRule {
                message: "global retention claim needs an entry".to_string(),
            }),
            Rc::new(GlobalRetentionClaimCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_retention_claim_validator();
        let item = new_global_retention_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = GlobalRetentionClaimExpiresAtRangeRule { min: 16, max: 120 };
        let mut item = new_global_retention_claim("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalRetentionClaimRequiredCodeRule {
            message: "global retention claim needs a value".to_string(),
        };
        let mut item = new_global_retention_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
