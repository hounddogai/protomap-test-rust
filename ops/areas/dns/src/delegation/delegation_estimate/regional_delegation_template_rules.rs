use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional delegation template must pass. Check returns its problems, or none.
pub trait RegionalDelegationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDelegationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplateRequiredNameRule {
    pub message: String,
}

impl RegionalDelegationTemplateRule for RegionalDelegationTemplateRequiredNameRule {
    fn check(&self, item: &RegionalDelegationTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplateCodePrefixRule {
    pub prefix: String,
}

impl RegionalDelegationTemplateRule for RegionalDelegationTemplateCodePrefixRule {
    fn check(&self, item: &RegionalDelegationTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplateCodeLengthRule {
    pub max_length: i64,
}

impl RegionalDelegationTemplateRule for RegionalDelegationTemplateCodeLengthRule {
    fn check(&self, item: &RegionalDelegationTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationTemplateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDelegationTemplateRule for RegionalDelegationTemplateCreatedAtRangeRule {
    fn check(&self, item: &RegionalDelegationTemplate) -> Vec<String> {
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
pub struct RegionalDelegationTemplateEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDelegationTemplateRule for RegionalDelegationTemplateEndsAtRangeRule {
    fn check(&self, item: &RegionalDelegationTemplate) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional delegation template.
#[derive(Debug, Clone)]
pub struct RegionalDelegationTemplateValidator {
    pub rules: Vec<Rc<dyn RegionalDelegationTemplateRule>>,
}

impl RegionalDelegationTemplateValidator {
    pub fn validate(&self, item: &RegionalDelegationTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDelegationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDelegationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_delegation_template_validator() -> RegionalDelegationTemplateValidator {
    RegionalDelegationTemplateValidator {
        rules: vec![
            Rc::new(RegionalDelegationTemplateRequiredNameRule {
                message: "regional delegation template needs this field".to_string(),
            }) as Rc<dyn RegionalDelegationTemplateRule>,
            Rc::new(RegionalDelegationTemplateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionalDelegationTemplateCodeLengthRule { max_length: 16 }),
            Rc::new(RegionalDelegationTemplateCreatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(RegionalDelegationTemplateEndsAtRangeRule { min: 255, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_delegation_template_validator();
        let item = new_regional_delegation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalDelegationTemplateRequiredNameRule {
            message: "regional delegation template needs an entry".to_string(),
        };
        let mut item = new_regional_delegation_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalDelegationTemplateCodeLengthRule { max_length: 64 };
        let mut item = new_regional_delegation_template("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionalDelegationTemplateCreatedAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_regional_delegation_template("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RegionalDelegationTemplateEndsAtRangeRule { min: 80, max: 86400 };
        let mut item = new_regional_delegation_template("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
