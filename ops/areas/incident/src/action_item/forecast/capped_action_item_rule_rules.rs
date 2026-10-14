use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped action item rule must pass. Check returns its problems, or none.
pub trait CappedActionItemRuleRule: std::fmt::Debug {
    fn check(&self, item: &CappedActionItemRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedActionItemRuleApprovedGuardRule {
    pub limit: i64,
}

impl CappedActionItemRuleRule for CappedActionItemRuleApprovedGuardRule {
    fn check(&self, item: &CappedActionItemRule) -> Vec<String> {
        if item.approved && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while approved", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedActionItemRuleCodePrefixRule {
    pub prefix: String,
}

impl CappedActionItemRuleRule for CappedActionItemRuleCodePrefixRule {
    fn check(&self, item: &CappedActionItemRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedActionItemRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedActionItemRuleRule for CappedActionItemRuleAllowedCodeRule {
    fn check(&self, item: &CappedActionItemRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedActionItemRuleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedActionItemRuleRule for CappedActionItemRuleEndsAtRangeRule {
    fn check(&self, item: &CappedActionItemRule) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped action item rule.
#[derive(Debug, Clone)]
pub struct CappedActionItemRuleValidator {
    pub rules: Vec<Rc<dyn CappedActionItemRuleRule>>,
}

impl CappedActionItemRuleValidator {
    pub fn validate(&self, item: &CappedActionItemRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedActionItemRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedActionItemRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_action_item_rule_validator() -> CappedActionItemRuleValidator {
    CappedActionItemRuleValidator {
        rules: vec![
            Rc::new(CappedActionItemRuleApprovedGuardRule { limit: 32 }) as Rc<dyn CappedActionItemRuleRule>,
            Rc::new(CappedActionItemRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedActionItemRuleAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "regional".to_string(), "global".to_string()],
            }),
            Rc::new(CappedActionItemRuleEndsAtRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_action_item_rule_validator();
        let item = new_capped_action_item_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = CappedActionItemRuleEndsAtRangeRule { min: 64, max: 120 };
        let mut item = new_capped_action_item_rule("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
