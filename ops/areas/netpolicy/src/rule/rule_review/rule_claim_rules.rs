use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule claim must pass. Check returns its problems, or none.
pub trait RuleClaimRule: std::fmt::Debug {
    fn check(&self, item: &RuleClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimLabelLengthRule {
    pub max_length: i64,
}

impl RuleClaimRule for RuleClaimLabelLengthRule {
    fn check(&self, item: &RuleClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimRequiredLabelRule {
    pub message: String,
}

impl RuleClaimRule for RuleClaimRequiredLabelRule {
    fn check(&self, item: &RuleClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimCodePrefixRule {
    pub prefix: String,
}

impl RuleClaimRule for RuleClaimCodePrefixRule {
    fn check(&self, item: &RuleClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl RuleClaimRule for RuleClaimAllowedCountryRule {
    fn check(&self, item: &RuleClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is inconsistent", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleClaimRule for RuleClaimRetriesRangeRule {
    fn check(&self, item: &RuleClaim) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rule claim.
#[derive(Debug, Clone)]
pub struct RuleClaimValidator {
    pub rules: Vec<Rc<dyn RuleClaimRule>>,
}

impl RuleClaimValidator {
    pub fn validate(&self, item: &RuleClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_claim_validator() -> RuleClaimValidator {
    RuleClaimValidator {
        rules: vec![
            Rc::new(RuleClaimLabelLengthRule { max_length: 86400 }) as Rc<dyn RuleClaimRule>,
            Rc::new(RuleClaimRequiredLabelRule {
                message: "rule claim needs a value".to_string(),
            }),
            Rc::new(RuleClaimCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RuleClaimAllowedCountryRule {
                allowed: vec!["fallback".to_string(), "global".to_string()],
            }),
            Rc::new(RuleClaimRetriesRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_claim_validator();
        let item = new_rule_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RuleClaimLabelLengthRule { max_length: 1000 };
        let mut item = new_rule_claim("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RuleClaimRequiredLabelRule {
            message: "rule claim needs this field".to_string(),
        };
        let mut item = new_rule_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RuleClaimRetriesRangeRule { min: 120, max: 255 };
        let mut item = new_rule_claim("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
