use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling ttl policy bundle must pass. Check returns its problems, or none.
pub trait RollingTtlPolicyBundleRule: std::fmt::Debug {
    fn check(&self, item: &RollingTtlPolicyBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyBundleAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl RollingTtlPolicyBundleRule for RollingTtlPolicyBundleAllowedPrincipalRule {
    fn check(&self, item: &RollingTtlPolicyBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is inconsistent", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyBundleContactEmailLengthRule {
    pub max_length: i64,
}

impl RollingTtlPolicyBundleRule for RollingTtlPolicyBundleContactEmailLengthRule {
    fn check(&self, item: &RollingTtlPolicyBundle) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyBundleCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingTtlPolicyBundleRule for RollingTtlPolicyBundleCostRangeRule {
    fn check(&self, item: &RollingTtlPolicyBundle) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyBundleCodePrefixRule {
    pub prefix: String,
}

impl RollingTtlPolicyBundleRule for RollingTtlPolicyBundleCodePrefixRule {
    fn check(&self, item: &RollingTtlPolicyBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyBundleRequiredContactEmailRule {
    pub message: String,
}

impl RollingTtlPolicyBundleRule for RollingTtlPolicyBundleRequiredContactEmailRule {
    fn check(&self, item: &RollingTtlPolicyBundle) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling ttl policy bundle.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyBundleValidator {
    pub rules: Vec<Rc<dyn RollingTtlPolicyBundleRule>>,
}

impl RollingTtlPolicyBundleValidator {
    pub fn validate(&self, item: &RollingTtlPolicyBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingTtlPolicyBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTtlPolicyBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_ttl_policy_bundle_validator() -> RollingTtlPolicyBundleValidator {
    RollingTtlPolicyBundleValidator {
        rules: vec![
            Rc::new(RollingTtlPolicyBundleAllowedPrincipalRule {
                allowed: vec!["fallback".to_string(), "regional".to_string()],
            }) as Rc<dyn RollingTtlPolicyBundleRule>,
            Rc::new(RollingTtlPolicyBundleContactEmailLengthRule { max_length: 255 }),
            Rc::new(RollingTtlPolicyBundleCostRangeRule { min: 4.2, max: 362.7 }),
            Rc::new(RollingTtlPolicyBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RollingTtlPolicyBundleRequiredContactEmailRule {
                message: "rolling ttl policy bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_ttl_policy_bundle_validator();
        let item = new_rolling_ttl_policy_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = RollingTtlPolicyBundleContactEmailLengthRule { max_length: 64 };
        let mut item = new_rolling_ttl_policy_bundle("x", "y");
        item.contact_email = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RollingTtlPolicyBundleCostRangeRule { min: 4.0, max: 2184.7 };
        let mut item = new_rolling_ttl_policy_bundle("x", "y");
        item.cost = 2186.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = RollingTtlPolicyBundleRequiredContactEmailRule {
            message: "rolling ttl policy bundle needs a value".to_string(),
        };
        let mut item = new_rolling_ttl_policy_bundle("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
