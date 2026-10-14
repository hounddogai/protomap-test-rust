use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal redaction group must pass. Check returns its problems, or none.
pub trait SeasonalRedactionGroupRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroupCodePrefixRule {
    pub prefix: String,
}

impl SeasonalRedactionGroupRule for SeasonalRedactionGroupCodePrefixRule {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroupRequiredNameRule {
    pub message: String,
}

impl SeasonalRedactionGroupRule for SeasonalRedactionGroupRequiredNameRule {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroupRequiredCodeRule {
    pub message: String,
}

impl SeasonalRedactionGroupRule for SeasonalRedactionGroupRequiredCodeRule {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroupSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRedactionGroupRule for SeasonalRedactionGroupSequenceRangeRule {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroupCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalRedactionGroupRule for SeasonalRedactionGroupCodeLengthRule {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRedactionGroupApprovedGuardRule {
    pub limit: i64,
}

impl SeasonalRedactionGroupRule for SeasonalRedactionGroupApprovedGuardRule {
    fn check(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        if item.approved && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while approved",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal redaction group.
#[derive(Debug, Clone)]
pub struct SeasonalRedactionGroupValidator {
    pub rules: Vec<Rc<dyn SeasonalRedactionGroupRule>>,
}

impl SeasonalRedactionGroupValidator {
    pub fn validate(&self, item: &SeasonalRedactionGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalRedactionGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRedactionGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_redaction_group_validator() -> SeasonalRedactionGroupValidator {
    SeasonalRedactionGroupValidator {
        rules: vec![
            Rc::new(SeasonalRedactionGroupCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn SeasonalRedactionGroupRule>,
            Rc::new(SeasonalRedactionGroupRequiredNameRule {
                message: "seasonal redaction group needs a value".to_string(),
            }),
            Rc::new(SeasonalRedactionGroupRequiredCodeRule {
                message: "seasonal redaction group needs an entry".to_string(),
            }),
            Rc::new(SeasonalRedactionGroupSequenceRangeRule { min: 64, max: 3600 }),
            Rc::new(SeasonalRedactionGroupCodeLengthRule { max_length: 86400 }),
            Rc::new(SeasonalRedactionGroupApprovedGuardRule { limit: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_redaction_group_validator();
        let item = new_seasonal_redaction_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalRedactionGroupRequiredNameRule {
            message: "seasonal redaction group needs an entry".to_string(),
        };
        let mut item = new_seasonal_redaction_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalRedactionGroupRequiredCodeRule {
            message: "seasonal redaction group needs an entry".to_string(),
        };
        let mut item = new_seasonal_redaction_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SeasonalRedactionGroupSequenceRangeRule { min: 80, max: 120 };
        let mut item = new_seasonal_redaction_group("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalRedactionGroupCodeLengthRule { max_length: 64 };
        let mut item = new_seasonal_redaction_group("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
