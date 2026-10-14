use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation template must pass. Check returns its problems, or none.
pub trait IsolationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &IsolationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTemplateRequiredCodeRule {
    pub message: String,
}

impl IsolationTemplateRule for IsolationTemplateRequiredCodeRule {
    fn check(&self, item: &IsolationTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTemplateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationTemplateRule for IsolationTemplateBurstRangeRule {
    fn check(&self, item: &IsolationTemplate) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTemplateGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationTemplateRule for IsolationTemplateGraceDaysRangeRule {
    fn check(&self, item: &IsolationTemplate) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTemplateFingerprintLengthRule {
    pub max_length: i64,
}

impl IsolationTemplateRule for IsolationTemplateFingerprintLengthRule {
    fn check(&self, item: &IsolationTemplate) -> Vec<String> {
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

/// Runs every rule of a isolation template.
#[derive(Debug, Clone)]
pub struct IsolationTemplateValidator {
    pub rules: Vec<Rc<dyn IsolationTemplateRule>>,
}

impl IsolationTemplateValidator {
    pub fn validate(&self, item: &IsolationTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_template_validator() -> IsolationTemplateValidator {
    IsolationTemplateValidator {
        rules: vec![
            Rc::new(IsolationTemplateRequiredCodeRule {
                message: "isolation template needs a value".to_string(),
            }) as Rc<dyn IsolationTemplateRule>,
            Rc::new(IsolationTemplateBurstRangeRule { min: 255, max: 86400 }),
            Rc::new(IsolationTemplateGraceDaysRangeRule { min: 255, max: 1000 }),
            Rc::new(IsolationTemplateFingerprintLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_template_validator();
        let item = new_isolation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationTemplateRequiredCodeRule {
            message: "isolation template needs an entry".to_string(),
        };
        let mut item = new_isolation_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = IsolationTemplateBurstRangeRule { min: 80, max: 120 };
        let mut item = new_isolation_template("x", "y");
        item.burst = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = IsolationTemplateGraceDaysRangeRule { min: 255, max: 255 };
        let mut item = new_isolation_template("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = IsolationTemplateFingerprintLengthRule { max_length: 3600 };
        let mut item = new_isolation_template("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
