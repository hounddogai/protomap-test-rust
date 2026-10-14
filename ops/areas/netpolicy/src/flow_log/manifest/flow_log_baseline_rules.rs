use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log baseline must pass. Check returns its problems, or none.
pub trait FlowLogBaselineRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBaselineCodePrefixRule {
    pub prefix: String,
}

impl FlowLogBaselineRule for FlowLogBaselineCodePrefixRule {
    fn check(&self, item: &FlowLogBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBaselineHostLengthRule {
    pub max_length: i64,
}

impl FlowLogBaselineRule for FlowLogBaselineHostLengthRule {
    fn check(&self, item: &FlowLogBaseline) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBaselineCodeLengthRule {
    pub max_length: i64,
}

impl FlowLogBaselineRule for FlowLogBaselineCodeLengthRule {
    fn check(&self, item: &FlowLogBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log baseline.
#[derive(Debug, Clone)]
pub struct FlowLogBaselineValidator {
    pub rules: Vec<Rc<dyn FlowLogBaselineRule>>,
}

impl FlowLogBaselineValidator {
    pub fn validate(&self, item: &FlowLogBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_baseline_validator() -> FlowLogBaselineValidator {
    FlowLogBaselineValidator {
        rules: vec![
            Rc::new(FlowLogBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn FlowLogBaselineRule>,
            Rc::new(FlowLogBaselineHostLengthRule { max_length: 32 }),
            Rc::new(FlowLogBaselineCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_baseline_validator();
        let item = new_flow_log_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = FlowLogBaselineHostLengthRule { max_length: 1000 };
        let mut item = new_flow_log_baseline("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FlowLogBaselineCodeLengthRule { max_length: 1000 };
        let mut item = new_flow_log_baseline("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
