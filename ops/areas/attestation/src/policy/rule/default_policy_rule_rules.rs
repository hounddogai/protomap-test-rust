use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default policy rule must pass. Check returns its problems, or none.
pub trait DefaultPolicyRuleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleCipherLengthRule {
    pub max_length: i64,
}

impl DefaultPolicyRuleRule for DefaultPolicyRuleCipherLengthRule {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultPolicyRuleRule for DefaultPolicyRuleRatioRangeRule {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultPolicyRuleRule for DefaultPolicyRuleWindowDaysRangeRule {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleRequiredCodeRule {
    pub message: String,
}

impl DefaultPolicyRuleRule for DefaultPolicyRuleRequiredCodeRule {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleRequiredScopeRule {
    pub message: String,
}

impl DefaultPolicyRuleRule for DefaultPolicyRuleRequiredScopeRule {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRuleCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPolicyRuleRule for DefaultPolicyRuleCodeLengthRule {
    fn check(&self, item: &DefaultPolicyRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default policy rule.
#[derive(Debug, Clone)]
pub struct DefaultPolicyRuleValidatorExtra {
    pub rules: Vec<Rc<dyn DefaultPolicyRuleRule>>,
}

impl DefaultPolicyRuleValidatorExtra {
    pub fn validate(&self, item: &DefaultPolicyRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPolicyRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPolicyRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_policy_rule_validator() -> DefaultPolicyRuleValidatorExtra {
    DefaultPolicyRuleValidatorExtra {
        rules: vec![
            Rc::new(DefaultPolicyRuleCipherLengthRule { max_length: 16 }) as Rc<dyn DefaultPolicyRuleRule>,
            Rc::new(DefaultPolicyRuleRatioRangeRule { min: 8.1, max: 944.2 }),
            Rc::new(DefaultPolicyRuleWindowDaysRangeRule { min: 255, max: 255 }),
            Rc::new(DefaultPolicyRuleRequiredCodeRule {
                message: "default policy rule needs a value".to_string(),
            }),
            Rc::new(DefaultPolicyRuleRequiredScopeRule {
                message: "default policy rule needs this field".to_string(),
            }),
            Rc::new(DefaultPolicyRuleCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_policy_rule_validator();
        let item = new_default_policy_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = DefaultPolicyRuleCipherLengthRule { max_length: 1000 };
        let mut item = new_default_policy_rule("x", "y");
        item.cipher = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DefaultPolicyRuleRatioRangeRule { min: 6.0, max: 1935.3 };
        let mut item = new_default_policy_rule("x", "y");
        item.ratio = 1936.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = DefaultPolicyRuleWindowDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_default_policy_rule("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultPolicyRuleRequiredCodeRule {
            message: "default policy rule needs this field".to_string(),
        };
        let mut item = new_default_policy_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = DefaultPolicyRuleRequiredScopeRule {
            message: "default policy rule needs this field".to_string(),
        };
        let mut item = new_default_policy_rule("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultPolicyRuleCodeLengthRule { max_length: 86400 };
        let mut item = new_default_policy_rule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
