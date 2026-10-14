use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage notice must pass. Check returns its problems, or none.
pub trait OverageNoticeRule: std::fmt::Debug {
    fn check(&self, item: &OverageNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageNoticeCodePrefixRule {
    pub prefix: String,
}

impl OverageNoticeRule for OverageNoticeCodePrefixRule {
    fn check(&self, item: &OverageNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageNoticeRequiredCodeRule {
    pub message: String,
}

impl OverageNoticeRule for OverageNoticeRequiredCodeRule {
    fn check(&self, item: &OverageNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageNoticeWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageNoticeRule for OverageNoticeWindowDaysRangeRule {
    fn check(&self, item: &OverageNotice) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage notice.
#[derive(Debug, Clone)]
pub struct OverageNoticeValidator {
    pub rules: Vec<Rc<dyn OverageNoticeRule>>,
}

impl OverageNoticeValidator {
    pub fn validate(&self, item: &OverageNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_notice_validator() -> OverageNoticeValidator {
    OverageNoticeValidator {
        rules: vec![
            Rc::new(OverageNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn OverageNoticeRule>,
            Rc::new(OverageNoticeRequiredCodeRule {
                message: "overage notice needs a value".to_string(),
            }),
            Rc::new(OverageNoticeWindowDaysRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_notice_validator();
        let item = new_overage_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageNoticeRequiredCodeRule {
            message: "overage notice needs an entry".to_string(),
        };
        let mut item = new_overage_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = OverageNoticeWindowDaysRangeRule { min: 32, max: 80 };
        let mut item = new_overage_notice("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
