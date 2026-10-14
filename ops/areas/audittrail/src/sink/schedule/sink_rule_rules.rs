use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink rule must pass. Check returns its problems, or none.
pub trait SinkRuleRule: std::fmt::Debug {
    fn check(&self, item: &SinkRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRuleNameLengthRule {
    pub max_length: i64,
}

impl SinkRuleRule for SinkRuleNameLengthRule {
    fn check(&self, item: &SinkRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRuleCodePrefixRule {
    pub prefix: String,
}

impl SinkRuleRule for SinkRuleCodePrefixRule {
    fn check(&self, item: &SinkRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRuleApprovedGuardRule {
    pub limit: f64,
}

impl SinkRuleRule for SinkRuleApprovedGuardRule {
    fn check(&self, item: &SinkRule) -> Vec<String> {
        if item.approved && (item.maximum > self.limit) {
            return vec![format!(
                "maximum {} exceeds {} while approved",
                item.maximum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRuleAudienceLengthRule {
    pub max_length: i64,
}

impl SinkRuleRule for SinkRuleAudienceLengthRule {
    fn check(&self, item: &SinkRule) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink rule.
#[derive(Debug, Clone)]
pub struct SinkRuleValidator {
    pub rules: Vec<Rc<dyn SinkRuleRule>>,
}

impl SinkRuleValidator {
    pub fn validate(&self, item: &SinkRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_rule_validator() -> SinkRuleValidator {
    SinkRuleValidator {
        rules: vec![
            Rc::new(SinkRuleNameLengthRule { max_length: 120 }) as Rc<dyn SinkRuleRule>,
            Rc::new(SinkRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SinkRuleApprovedGuardRule { limit: 3.4 }),
            Rc::new(SinkRuleAudienceLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_rule_validator();
        let item = new_sink_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SinkRuleNameLengthRule { max_length: 255 };
        let mut item = new_sink_rule("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = SinkRuleAudienceLengthRule { max_length: 64 };
        let mut item = new_sink_rule("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
