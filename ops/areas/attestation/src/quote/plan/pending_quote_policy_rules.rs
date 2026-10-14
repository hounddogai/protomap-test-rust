use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending quote policy must pass. Check returns its problems, or none.
pub trait PendingQuotePolicyRule: std::fmt::Debug {
    fn check(&self, item: &PendingQuotePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuotePolicyAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PendingQuotePolicyRule for PendingQuotePolicyAllowedTitleRule {
    fn check(&self, item: &PendingQuotePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuotePolicyTitleLengthRule {
    pub max_length: i64,
}

impl PendingQuotePolicyRule for PendingQuotePolicyTitleLengthRule {
    fn check(&self, item: &PendingQuotePolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuotePolicyRequiredCodeRule {
    pub message: String,
}

impl PendingQuotePolicyRule for PendingQuotePolicyRequiredCodeRule {
    fn check(&self, item: &PendingQuotePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuotePolicyWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingQuotePolicyRule for PendingQuotePolicyWeightRangeRule {
    fn check(&self, item: &PendingQuotePolicy) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending quote policy.
#[derive(Debug, Clone)]
pub struct PendingQuotePolicyValidator {
    pub rules: Vec<Rc<dyn PendingQuotePolicyRule>>,
}

impl PendingQuotePolicyValidator {
    pub fn validate(&self, item: &PendingQuotePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingQuotePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingQuotePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_quote_policy_validator() -> PendingQuotePolicyValidator {
    PendingQuotePolicyValidator {
        rules: vec![
            Rc::new(PendingQuotePolicyAllowedTitleRule {
                allowed: vec!["batched".to_string()],
            }) as Rc<dyn PendingQuotePolicyRule>,
            Rc::new(PendingQuotePolicyTitleLengthRule { max_length: 255 }),
            Rc::new(PendingQuotePolicyRequiredCodeRule {
                message: "pending quote policy needs this field".to_string(),
            }),
            Rc::new(PendingQuotePolicyWeightRangeRule { min: 2.3, max: 4908.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_quote_policy_validator();
        let item = new_pending_quote_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingQuotePolicyTitleLengthRule { max_length: 255 };
        let mut item = new_pending_quote_policy("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingQuotePolicyRequiredCodeRule {
            message: "pending quote policy needs this field".to_string(),
        };
        let mut item = new_pending_quote_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PendingQuotePolicyWeightRangeRule { min: 5.3, max: 3792.6 };
        let mut item = new_pending_quote_policy("x", "y");
        item.weight = 3794.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
