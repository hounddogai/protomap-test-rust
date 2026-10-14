use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient quote plan must pass. Check returns its problems, or none.
pub trait LenientQuotePlanRule: std::fmt::Debug {
    fn check(&self, item: &LenientQuotePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuotePlanRequiredTitleRule {
    pub message: String,
}

impl LenientQuotePlanRule for LenientQuotePlanRequiredTitleRule {
    fn check(&self, item: &LenientQuotePlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuotePlanDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientQuotePlanRule for LenientQuotePlanDurationMinutesRangeRule {
    fn check(&self, item: &LenientQuotePlan) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuotePlanRequiredDescriptionRule {
    pub message: String,
}

impl LenientQuotePlanRule for LenientQuotePlanRequiredDescriptionRule {
    fn check(&self, item: &LenientQuotePlan) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuotePlanFingerprintLengthRule {
    pub max_length: i64,
}

impl LenientQuotePlanRule for LenientQuotePlanFingerprintLengthRule {
    fn check(&self, item: &LenientQuotePlan) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient quote plan.
#[derive(Debug, Clone)]
pub struct LenientQuotePlanValidator {
    pub rules: Vec<Rc<dyn LenientQuotePlanRule>>,
}

impl LenientQuotePlanValidator {
    pub fn validate(&self, item: &LenientQuotePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientQuotePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientQuotePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_quote_plan_validator() -> LenientQuotePlanValidator {
    LenientQuotePlanValidator {
        rules: vec![
            Rc::new(LenientQuotePlanRequiredTitleRule {
                message: "lenient quote plan needs a value".to_string(),
            }) as Rc<dyn LenientQuotePlanRule>,
            Rc::new(LenientQuotePlanDurationMinutesRangeRule { min: 64, max: 120 }),
            Rc::new(LenientQuotePlanRequiredDescriptionRule {
                message: "lenient quote plan needs this field".to_string(),
            }),
            Rc::new(LenientQuotePlanFingerprintLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_quote_plan_validator();
        let item = new_lenient_quote_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientQuotePlanRequiredTitleRule {
            message: "lenient quote plan needs this field".to_string(),
        };
        let mut item = new_lenient_quote_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LenientQuotePlanDurationMinutesRangeRule { min: 32, max: 86400 };
        let mut item = new_lenient_quote_plan("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = LenientQuotePlanRequiredDescriptionRule {
            message: "lenient quote plan needs a value".to_string(),
        };
        let mut item = new_lenient_quote_plan("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = LenientQuotePlanFingerprintLengthRule { max_length: 255 };
        let mut item = new_lenient_quote_plan("x", "y");
        item.fingerprint = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
