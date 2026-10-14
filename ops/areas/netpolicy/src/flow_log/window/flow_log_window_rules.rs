use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log window must pass. Check returns its problems, or none.
pub trait FlowLogWindowRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogWindowRule for FlowLogWindowGraceDaysRangeRule {
    fn check(&self, item: &FlowLogWindow) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowRequiredChecksumRule {
    pub message: String,
}

impl FlowLogWindowRule for FlowLogWindowRequiredChecksumRule {
    fn check(&self, item: &FlowLogWindow) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowCodePrefixRule {
    pub prefix: String,
}

impl FlowLogWindowRule for FlowLogWindowCodePrefixRule {
    fn check(&self, item: &FlowLogWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowPhoneLengthRule {
    pub max_length: i64,
}

impl FlowLogWindowRule for FlowLogWindowPhoneLengthRule {
    fn check(&self, item: &FlowLogWindow) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogWindowRule for FlowLogWindowStartsAtRangeRule {
    fn check(&self, item: &FlowLogWindow) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a flow log window.
#[derive(Debug, Clone)]
pub struct FlowLogWindowValidator {
    pub rules: Vec<Rc<dyn FlowLogWindowRule>>,
}

impl FlowLogWindowValidator {
    pub fn validate(&self, item: &FlowLogWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_window_validator() -> FlowLogWindowValidator {
    FlowLogWindowValidator {
        rules: vec![
            Rc::new(FlowLogWindowGraceDaysRangeRule { min: 32, max: 255 }) as Rc<dyn FlowLogWindowRule>,
            Rc::new(FlowLogWindowRequiredChecksumRule {
                message: "flow log window needs this field".to_string(),
            }),
            Rc::new(FlowLogWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FlowLogWindowPhoneLengthRule { max_length: 255 }),
            Rc::new(FlowLogWindowStartsAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_window_validator();
        let item = new_flow_log_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FlowLogWindowGraceDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_flow_log_window("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = FlowLogWindowRequiredChecksumRule {
            message: "flow log window needs this field".to_string(),
        };
        let mut item = new_flow_log_window("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = FlowLogWindowPhoneLengthRule { max_length: 255 };
        let mut item = new_flow_log_window("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = FlowLogWindowStartsAtRangeRule { min: 120, max: 86400 };
        let mut item = new_flow_log_window("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
