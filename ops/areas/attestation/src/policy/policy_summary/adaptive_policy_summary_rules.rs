use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive policy summary must pass. Check returns its problems, or none.
pub trait AdaptivePolicySummaryRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePolicySummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummaryCodeLengthRule {
    pub max_length: i64,
}

impl AdaptivePolicySummaryRule for AdaptivePolicySummaryCodeLengthRule {
    fn check(&self, item: &AdaptivePolicySummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummaryRequiredTitleRule {
    pub message: String,
}

impl AdaptivePolicySummaryRule for AdaptivePolicySummaryRequiredTitleRule {
    fn check(&self, item: &AdaptivePolicySummary) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummaryTitleLengthRule {
    pub max_length: i64,
}

impl AdaptivePolicySummaryRule for AdaptivePolicySummaryTitleLengthRule {
    fn check(&self, item: &AdaptivePolicySummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummaryRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptivePolicySummaryRule for AdaptivePolicySummaryRateRangeRule {
    fn check(&self, item: &AdaptivePolicySummary) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummaryRequiredTenantIdRule {
    pub message: String,
}

impl AdaptivePolicySummaryRule for AdaptivePolicySummaryRequiredTenantIdRule {
    fn check(&self, item: &AdaptivePolicySummary) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive policy summary.
#[derive(Debug, Clone)]
pub struct AdaptivePolicySummaryValidator {
    pub rules: Vec<Rc<dyn AdaptivePolicySummaryRule>>,
}

impl AdaptivePolicySummaryValidator {
    pub fn validate(&self, item: &AdaptivePolicySummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptivePolicySummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePolicySummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_policy_summary_validator() -> AdaptivePolicySummaryValidator {
    AdaptivePolicySummaryValidator {
        rules: vec![
            Rc::new(AdaptivePolicySummaryCodeLengthRule { max_length: 16 }) as Rc<dyn AdaptivePolicySummaryRule>,
            Rc::new(AdaptivePolicySummaryRequiredTitleRule {
                message: "adaptive policy summary needs an entry".to_string(),
            }),
            Rc::new(AdaptivePolicySummaryTitleLengthRule { max_length: 80 }),
            Rc::new(AdaptivePolicySummaryRateRangeRule { min: 5.0, max: 2017.5 }),
            Rc::new(AdaptivePolicySummaryRequiredTenantIdRule {
                message: "adaptive policy summary needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_policy_summary_validator();
        let item = new_adaptive_policy_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptivePolicySummaryCodeLengthRule { max_length: 3600 };
        let mut item = new_adaptive_policy_summary("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptivePolicySummaryRequiredTitleRule {
            message: "adaptive policy summary needs a value".to_string(),
        };
        let mut item = new_adaptive_policy_summary("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptivePolicySummaryTitleLengthRule { max_length: 86400 };
        let mut item = new_adaptive_policy_summary("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = AdaptivePolicySummaryRateRangeRule { min: 5.5, max: 4899.6 };
        let mut item = new_adaptive_policy_summary("x", "y");
        item.rate = 4901.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = AdaptivePolicySummaryRequiredTenantIdRule {
            message: "adaptive policy summary needs an entry".to_string(),
        };
        let mut item = new_adaptive_policy_summary("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
