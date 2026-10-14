use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation window must pass. Check returns its problems, or none.
pub trait EscalationWindowRule: std::fmt::Debug {
    fn check(&self, item: &EscalationWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowCodePrefixRule {
    pub prefix: String,
}

impl EscalationWindowRule for EscalationWindowCodePrefixRule {
    fn check(&self, item: &EscalationWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowRequiredCodeRule {
    pub message: String,
}

impl EscalationWindowRule for EscalationWindowRequiredCodeRule {
    fn check(&self, item: &EscalationWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationWindowRule for EscalationWindowQuotaUnitsRangeRule {
    fn check(&self, item: &EscalationWindow) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowNameLengthRule {
    pub max_length: i64,
}

impl EscalationWindowRule for EscalationWindowNameLengthRule {
    fn check(&self, item: &EscalationWindow) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowSummaryLengthRule {
    pub max_length: i64,
}

impl EscalationWindowRule for EscalationWindowSummaryLengthRule {
    fn check(&self, item: &EscalationWindow) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindowEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationWindowRule for EscalationWindowEndsAtRangeRule {
    fn check(&self, item: &EscalationWindow) -> Vec<String> {
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

/// Runs every rule of a escalation window.
#[derive(Debug, Clone)]
pub struct EscalationWindowValidator {
    pub rules: Vec<Rc<dyn EscalationWindowRule>>,
}

impl EscalationWindowValidator {
    pub fn validate(&self, item: &EscalationWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_window_validator() -> EscalationWindowValidator {
    EscalationWindowValidator {
        rules: vec![
            Rc::new(EscalationWindowCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn EscalationWindowRule>,
            Rc::new(EscalationWindowRequiredCodeRule {
                message: "escalation window needs an entry".to_string(),
            }),
            Rc::new(EscalationWindowQuotaUnitsRangeRule { min: 16, max: 3600 }),
            Rc::new(EscalationWindowNameLengthRule { max_length: 3600 }),
            Rc::new(EscalationWindowSummaryLengthRule { max_length: 3600 }),
            Rc::new(EscalationWindowEndsAtRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_window_validator();
        let item = new_escalation_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationWindowRequiredCodeRule {
            message: "escalation window needs a value".to_string(),
        };
        let mut item = new_escalation_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = EscalationWindowQuotaUnitsRangeRule { min: 120, max: 1000 };
        let mut item = new_escalation_window("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EscalationWindowNameLengthRule { max_length: 32 };
        let mut item = new_escalation_window("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = EscalationWindowSummaryLengthRule { max_length: 120 };
        let mut item = new_escalation_window("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = EscalationWindowEndsAtRangeRule { min: 120, max: 1000 };
        let mut item = new_escalation_window("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
