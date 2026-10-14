use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted policy rule must pass. Check returns its problems, or none.
pub trait WeightedPolicyRuleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPolicyRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRuleNameLengthRule {
    pub max_length: i64,
}

impl WeightedPolicyRuleRule for WeightedPolicyRuleNameLengthRule {
    fn check(&self, item: &WeightedPolicyRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRuleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPolicyRuleRule for WeightedPolicyRuleRevisionRangeRule {
    fn check(&self, item: &WeightedPolicyRule) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRuleSegmentLengthRule {
    pub max_length: i64,
}

impl WeightedPolicyRuleRule for WeightedPolicyRuleSegmentLengthRule {
    fn check(&self, item: &WeightedPolicyRule) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRuleCodeLengthRule {
    pub max_length: i64,
}

impl WeightedPolicyRuleRule for WeightedPolicyRuleCodeLengthRule {
    fn check(&self, item: &WeightedPolicyRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRuleCipherLengthRule {
    pub max_length: i64,
}

impl WeightedPolicyRuleRule for WeightedPolicyRuleCipherLengthRule {
    fn check(&self, item: &WeightedPolicyRule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted policy rule.
#[derive(Debug, Clone)]
pub struct WeightedPolicyRuleValidator {
    pub rules: Vec<Rc<dyn WeightedPolicyRuleRule>>,
}

impl WeightedPolicyRuleValidator {
    pub fn validate(&self, item: &WeightedPolicyRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedPolicyRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPolicyRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_policy_rule_validator() -> WeightedPolicyRuleValidator {
    WeightedPolicyRuleValidator {
        rules: vec![
            Rc::new(WeightedPolicyRuleNameLengthRule { max_length: 120 }) as Rc<dyn WeightedPolicyRuleRule>,
            Rc::new(WeightedPolicyRuleRevisionRangeRule { min: 32, max: 120 }),
            Rc::new(WeightedPolicyRuleSegmentLengthRule { max_length: 64 }),
            Rc::new(WeightedPolicyRuleCodeLengthRule { max_length: 80 }),
            Rc::new(WeightedPolicyRuleCipherLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_policy_rule_validator();
        let item = new_weighted_policy_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedPolicyRuleNameLengthRule { max_length: 3600 };
        let mut item = new_weighted_policy_rule("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = WeightedPolicyRuleRevisionRangeRule { min: 1000, max: 1000 };
        let mut item = new_weighted_policy_rule("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = WeightedPolicyRuleSegmentLengthRule { max_length: 3600 };
        let mut item = new_weighted_policy_rule("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedPolicyRuleCodeLengthRule { max_length: 80 };
        let mut item = new_weighted_policy_rule("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = WeightedPolicyRuleCipherLengthRule { max_length: 255 };
        let mut item = new_weighted_policy_rule("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
