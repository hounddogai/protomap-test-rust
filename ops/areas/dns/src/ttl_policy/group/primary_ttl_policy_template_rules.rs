use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary ttl policy template must pass. Check returns its problems, or none.
pub trait PrimaryTtlPolicyTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTtlPolicyTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyTemplateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryTtlPolicyTemplateRule for PrimaryTtlPolicyTemplateLimitRangeRule {
    fn check(&self, item: &PrimaryTtlPolicyTemplate) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyTemplateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryTtlPolicyTemplateRule for PrimaryTtlPolicyTemplateCreatedAtRangeRule {
    fn check(&self, item: &PrimaryTtlPolicyTemplate) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyTemplateRequiredCodeRule {
    pub message: String,
}

impl PrimaryTtlPolicyTemplateRule for PrimaryTtlPolicyTemplateRequiredCodeRule {
    fn check(&self, item: &PrimaryTtlPolicyTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary ttl policy template.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyTemplateValidator {
    pub rules: Vec<Rc<dyn PrimaryTtlPolicyTemplateRule>>,
}

impl PrimaryTtlPolicyTemplateValidator {
    pub fn validate(&self, item: &PrimaryTtlPolicyTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryTtlPolicyTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTtlPolicyTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_ttl_policy_template_validator() -> PrimaryTtlPolicyTemplateValidator {
    PrimaryTtlPolicyTemplateValidator {
        rules: vec![
            Rc::new(PrimaryTtlPolicyTemplateLimitRangeRule { min: 120, max: 255 })
                as Rc<dyn PrimaryTtlPolicyTemplateRule>,
            Rc::new(PrimaryTtlPolicyTemplateCreatedAtRangeRule { min: 255, max: 3600 }),
            Rc::new(PrimaryTtlPolicyTemplateRequiredCodeRule {
                message: "primary ttl policy template needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_ttl_policy_template_validator();
        let item = new_primary_ttl_policy_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PrimaryTtlPolicyTemplateLimitRangeRule { min: 120, max: 255 };
        let mut item = new_primary_ttl_policy_template("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PrimaryTtlPolicyTemplateCreatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_primary_ttl_policy_template("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryTtlPolicyTemplateRequiredCodeRule {
            message: "primary ttl policy template needs a value".to_string(),
        };
        let mut item = new_primary_ttl_policy_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
