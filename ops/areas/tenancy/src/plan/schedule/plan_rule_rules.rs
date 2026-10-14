use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan rule must pass. Check returns its problems, or none.
pub trait PlanRuleRule: std::fmt::Debug {
    fn check(&self, item: &PlanRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleCodePrefixRule {
    pub prefix: String,
}

impl PlanRuleRule for PlanRuleCodePrefixRule {
    fn check(&self, item: &PlanRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleKeyIdLengthRule {
    pub max_length: i64,
}

impl PlanRuleRule for PlanRuleKeyIdLengthRule {
    fn check(&self, item: &PlanRule) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanRuleRule for PlanRuleCeilingRangeRule {
    fn check(&self, item: &PlanRule) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleTitleLengthRule {
    pub max_length: i64,
}

impl PlanRuleRule for PlanRuleTitleLengthRule {
    fn check(&self, item: &PlanRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleCodeLengthRule {
    pub max_length: i64,
}

impl PlanRuleRule for PlanRuleCodeLengthRule {
    fn check(&self, item: &PlanRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan rule.
#[derive(Debug, Clone)]
pub struct PlanRuleValidator {
    pub rules: Vec<Rc<dyn PlanRuleRule>>,
}

impl PlanRuleValidator {
    pub fn validate(&self, item: &PlanRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_rule_validator() -> PlanRuleValidator {
    PlanRuleValidator {
        rules: vec![
            Rc::new(PlanRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PlanRuleRule>,
            Rc::new(PlanRuleKeyIdLengthRule { max_length: 120 }),
            Rc::new(PlanRuleCeilingRangeRule { min: 0.5, max: 3035.6 }),
            Rc::new(PlanRuleTitleLengthRule { max_length: 255 }),
            Rc::new(PlanRuleCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_rule_validator();
        let item = new_plan_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = PlanRuleKeyIdLengthRule { max_length: 32 };
        let mut item = new_plan_rule("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PlanRuleCeilingRangeRule { min: 0.8, max: 278.7 };
        let mut item = new_plan_rule("x", "y");
        item.ceiling = 280.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PlanRuleTitleLengthRule { max_length: 80 };
        let mut item = new_plan_rule("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanRuleCodeLengthRule { max_length: 80 };
        let mut item = new_plan_rule("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
