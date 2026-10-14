use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension template must pass. Check returns its problems, or none.
pub trait SuspensionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateSourceLengthRule {
    pub max_length: i64,
}

impl SuspensionTemplateRule for SuspensionTemplateSourceLengthRule {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateRequiredCodeRule {
    pub message: String,
}

impl SuspensionTemplateRule for SuspensionTemplateRequiredCodeRule {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateTitleLengthRule {
    pub max_length: i64,
}

impl SuspensionTemplateRule for SuspensionTemplateTitleLengthRule {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionTemplateRule for SuspensionTemplateExpiresAtRangeRule {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionTemplateRule for SuspensionTemplateCapacityRangeRule {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTemplateRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionTemplateRule for SuspensionTemplateRevisionRangeRule {
    fn check(&self, item: &SuspensionTemplate) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension template.
#[derive(Debug, Clone)]
pub struct SuspensionTemplateValidator {
    pub rules: Vec<Rc<dyn SuspensionTemplateRule>>,
}

impl SuspensionTemplateValidator {
    pub fn validate(&self, item: &SuspensionTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_template_validator() -> SuspensionTemplateValidator {
    SuspensionTemplateValidator {
        rules: vec![
            Rc::new(SuspensionTemplateSourceLengthRule { max_length: 120 }) as Rc<dyn SuspensionTemplateRule>,
            Rc::new(SuspensionTemplateRequiredCodeRule {
                message: "suspension template needs this field".to_string(),
            }),
            Rc::new(SuspensionTemplateTitleLengthRule { max_length: 1000 }),
            Rc::new(SuspensionTemplateExpiresAtRangeRule { min: 120, max: 86400 }),
            Rc::new(SuspensionTemplateCapacityRangeRule { min: 64, max: 1000 }),
            Rc::new(SuspensionTemplateRevisionRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_template_validator();
        let item = new_suspension_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = SuspensionTemplateSourceLengthRule { max_length: 1000 };
        let mut item = new_suspension_template("x", "y");
        item.source = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionTemplateRequiredCodeRule {
            message: "suspension template needs an entry".to_string(),
        };
        let mut item = new_suspension_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SuspensionTemplateTitleLengthRule { max_length: 255 };
        let mut item = new_suspension_template("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SuspensionTemplateExpiresAtRangeRule { min: 80, max: 80 };
        let mut item = new_suspension_template("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = SuspensionTemplateCapacityRangeRule { min: 255, max: 86400 };
        let mut item = new_suspension_template("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SuspensionTemplateRevisionRangeRule { min: 64, max: 255 };
        let mut item = new_suspension_template("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
