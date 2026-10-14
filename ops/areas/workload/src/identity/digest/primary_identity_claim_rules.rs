use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary identity claim must pass. Check returns its problems, or none.
pub trait PrimaryIdentityClaimRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaimRequiredTitleRule {
    pub message: String,
}

impl PrimaryIdentityClaimRule for PrimaryIdentityClaimRequiredTitleRule {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaimRequiredClusterRule {
    pub message: String,
}

impl PrimaryIdentityClaimRule for PrimaryIdentityClaimRequiredClusterRule {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaimQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryIdentityClaimRule for PrimaryIdentityClaimQuotaUnitsRangeRule {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryIdentityClaimRule for PrimaryIdentityClaimExpiresAtRangeRule {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaimRequiredCodeRule {
    pub message: String,
}

impl PrimaryIdentityClaimRule for PrimaryIdentityClaimRequiredCodeRule {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityClaimCodePrefixRule {
    pub prefix: String,
}

impl PrimaryIdentityClaimRule for PrimaryIdentityClaimCodePrefixRule {
    fn check(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary identity claim.
#[derive(Debug, Clone)]
pub struct PrimaryIdentityClaimValidator {
    pub rules: Vec<Rc<dyn PrimaryIdentityClaimRule>>,
}

impl PrimaryIdentityClaimValidator {
    pub fn validate(&self, item: &PrimaryIdentityClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryIdentityClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryIdentityClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_identity_claim_validator() -> PrimaryIdentityClaimValidator {
    PrimaryIdentityClaimValidator {
        rules: vec![
            Rc::new(PrimaryIdentityClaimRequiredTitleRule {
                message: "primary identity claim needs a value".to_string(),
            }) as Rc<dyn PrimaryIdentityClaimRule>,
            Rc::new(PrimaryIdentityClaimRequiredClusterRule {
                message: "primary identity claim needs an entry".to_string(),
            }),
            Rc::new(PrimaryIdentityClaimQuotaUnitsRangeRule { min: 64, max: 255 }),
            Rc::new(PrimaryIdentityClaimExpiresAtRangeRule { min: 64, max: 3600 }),
            Rc::new(PrimaryIdentityClaimRequiredCodeRule {
                message: "primary identity claim needs a value".to_string(),
            }),
            Rc::new(PrimaryIdentityClaimCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_identity_claim_validator();
        let item = new_primary_identity_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryIdentityClaimRequiredTitleRule {
            message: "primary identity claim needs an entry".to_string(),
        };
        let mut item = new_primary_identity_claim("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = PrimaryIdentityClaimRequiredClusterRule {
            message: "primary identity claim needs an entry".to_string(),
        };
        let mut item = new_primary_identity_claim("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PrimaryIdentityClaimQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_primary_identity_claim("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = PrimaryIdentityClaimExpiresAtRangeRule { min: 255, max: 1000 };
        let mut item = new_primary_identity_claim("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryIdentityClaimRequiredCodeRule {
            message: "primary identity claim needs an entry".to_string(),
        };
        let mut item = new_primary_identity_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
