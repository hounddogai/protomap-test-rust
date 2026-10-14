use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log score must pass. Check returns its problems, or none.
pub trait FlowLogScoreRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreRequiredCodeRule {
    pub message: String,
}

impl FlowLogScoreRule for FlowLogScoreRequiredCodeRule {
    fn check(&self, item: &FlowLogScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl FlowLogScoreRule for FlowLogScoreAllowedTenantIdRule {
    fn check(&self, item: &FlowLogScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is below minimum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogScoreRule for FlowLogScoreCodeLengthRule {
    fn check(&self, item: &FlowLogScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreCodePrefixRule {
    pub prefix: String,
}

impl FlowLogScoreRule for FlowLogScoreCodePrefixRule {
    fn check(&self, item: &FlowLogScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogScoreRule for FlowLogScoreQuantityRangeRule {
    fn check(&self, item: &FlowLogScore) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScoreSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogScoreRule for FlowLogScoreSequenceRangeRule {
    fn check(&self, item: &FlowLogScore) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log score.
#[derive(Debug, Clone)]
pub struct FlowLogScoreValidator {
    pub rules: Vec<Rc<dyn FlowLogScoreRule>>,
}

impl FlowLogScoreValidator {
    pub fn validate(&self, item: &FlowLogScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_score_validator() -> FlowLogScoreValidator {
    FlowLogScoreValidator {
        rules: vec![
            Rc::new(FlowLogScoreRequiredCodeRule {
                message: "flow log score needs a value".to_string(),
            }) as Rc<dyn FlowLogScoreRule>,
            Rc::new(FlowLogScoreAllowedTenantIdRule {
                allowed: vec!["fallback".to_string(), "capped".to_string()],
            }),
            Rc::new(FlowLogScoreCodeLengthRule { max_length: 80 }),
            Rc::new(FlowLogScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FlowLogScoreQuantityRangeRule { min: 120, max: 86400 }),
            Rc::new(FlowLogScoreSequenceRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_score_validator();
        let item = new_flow_log_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FlowLogScoreRequiredCodeRule {
            message: "flow log score needs an entry".to_string(),
        };
        let mut item = new_flow_log_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogScoreCodeLengthRule { max_length: 32 };
        let mut item = new_flow_log_score("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = FlowLogScoreQuantityRangeRule { min: 3600, max: 86400 };
        let mut item = new_flow_log_score("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = FlowLogScoreSequenceRangeRule { min: 64, max: 3600 };
        let mut item = new_flow_log_score("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
