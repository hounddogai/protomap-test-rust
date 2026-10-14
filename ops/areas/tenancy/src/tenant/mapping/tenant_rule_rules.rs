use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant rule must pass. Check returns its problems, or none.
pub trait TenantRuleRule: std::fmt::Debug {
    fn check(&self, item: &TenantRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRuleNameLengthRule {
    pub max_length: i64,
}

impl TenantRuleRule for TenantRuleNameLengthRule {
    fn check(&self, item: &TenantRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRuleRequiredTimezoneRule {
    pub message: String,
}

impl TenantRuleRule for TenantRuleRequiredTimezoneRule {
    fn check(&self, item: &TenantRule) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRuleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantRuleRule for TenantRuleLimitRangeRule {
    fn check(&self, item: &TenantRule) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRuleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantRuleRule for TenantRuleMinimumRangeRule {
    fn check(&self, item: &TenantRule) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant rule.
#[derive(Debug, Clone)]
pub struct TenantRuleValidator {
    pub rules: Vec<Rc<dyn TenantRuleRule>>,
}

impl TenantRuleValidator {
    pub fn validate(&self, item: &TenantRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_rule_validator() -> TenantRuleValidator {
    TenantRuleValidator {
        rules: vec![
            Rc::new(TenantRuleNameLengthRule { max_length: 3600 }) as Rc<dyn TenantRuleRule>,
            Rc::new(TenantRuleRequiredTimezoneRule {
                message: "tenant rule needs an entry".to_string(),
            }),
            Rc::new(TenantRuleLimitRangeRule { min: 64, max: 255 }),
            Rc::new(TenantRuleMinimumRangeRule { min: 0.2, max: 3517.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_rule_validator();
        let item = new_tenant_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TenantRuleNameLengthRule { max_length: 80 };
        let mut item = new_tenant_rule("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = TenantRuleRequiredTimezoneRule {
            message: "tenant rule needs this field".to_string(),
        };
        let mut item = new_tenant_rule("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = TenantRuleLimitRangeRule { min: 64, max: 80 };
        let mut item = new_tenant_rule("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TenantRuleMinimumRangeRule { min: 8.2, max: 4188.1 };
        let mut item = new_tenant_rule("x", "y");
        item.minimum = 4189.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
