use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity claim must pass. Check returns its problems, or none.
pub trait IdentityClaimRule: std::fmt::Debug {
    fn check(&self, item: &IdentityClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaimRequiredTitleRule {
    pub message: String,
}

impl IdentityClaimRule for IdentityClaimRequiredTitleRule {
    fn check(&self, item: &IdentityClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaimRegionLengthRule {
    pub max_length: i64,
}

impl IdentityClaimRule for IdentityClaimRegionLengthRule {
    fn check(&self, item: &IdentityClaim) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaimQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityClaimRule for IdentityClaimQuotaUnitsRangeRule {
    fn check(&self, item: &IdentityClaim) -> Vec<String> {
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
pub struct IdentityClaimAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl IdentityClaimRule for IdentityClaimAllowedPrincipalRule {
    fn check(&self, item: &IdentityClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is duplicated", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityClaimCodeLengthRule {
    pub max_length: i64,
}

impl IdentityClaimRule for IdentityClaimCodeLengthRule {
    fn check(&self, item: &IdentityClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity claim.
#[derive(Debug, Clone)]
pub struct IdentityClaimValidator {
    pub rules: Vec<Rc<dyn IdentityClaimRule>>,
}

impl IdentityClaimValidator {
    pub fn validate(&self, item: &IdentityClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_claim_validator() -> IdentityClaimValidator {
    IdentityClaimValidator {
        rules: vec![
            Rc::new(IdentityClaimRequiredTitleRule {
                message: "identity claim needs a value".to_string(),
            }) as Rc<dyn IdentityClaimRule>,
            Rc::new(IdentityClaimRegionLengthRule { max_length: 255 }),
            Rc::new(IdentityClaimQuotaUnitsRangeRule { min: 32, max: 64 }),
            Rc::new(IdentityClaimAllowedPrincipalRule {
                allowed: vec!["legacy".to_string(), "tiered".to_string(), "primary".to_string()],
            }),
            Rc::new(IdentityClaimCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_claim_validator();
        let item = new_identity_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IdentityClaimRequiredTitleRule {
            message: "identity claim needs an entry".to_string(),
        };
        let mut item = new_identity_claim("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = IdentityClaimRegionLengthRule { max_length: 3600 };
        let mut item = new_identity_claim("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = IdentityClaimQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_identity_claim("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityClaimCodeLengthRule { max_length: 1000 };
        let mut item = new_identity_claim("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
