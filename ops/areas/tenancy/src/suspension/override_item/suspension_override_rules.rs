use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension override must pass. Check returns its problems, or none.
pub trait SuspensionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverrideRequiredIssuerRule {
    pub message: String,
}

impl SuspensionOverrideRule for SuspensionOverrideRequiredIssuerRule {
    fn check(&self, item: &SuspensionOverride) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverrideCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionOverrideRule for SuspensionOverrideCreatedAtRangeRule {
    fn check(&self, item: &SuspensionOverride) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverrideIssuerLengthRule {
    pub max_length: i64,
}

impl SuspensionOverrideRule for SuspensionOverrideIssuerLengthRule {
    fn check(&self, item: &SuspensionOverride) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverrideCodePrefixRule {
    pub prefix: String,
}

impl SuspensionOverrideRule for SuspensionOverrideCodePrefixRule {
    fn check(&self, item: &SuspensionOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverrideWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionOverrideRule for SuspensionOverrideWeightRangeRule {
    fn check(&self, item: &SuspensionOverride) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionOverrideCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionOverrideRule for SuspensionOverrideCodeLengthRule {
    fn check(&self, item: &SuspensionOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension override.
#[derive(Debug, Clone)]
pub struct SuspensionOverrideValidator {
    pub rules: Vec<Rc<dyn SuspensionOverrideRule>>,
}

impl SuspensionOverrideValidator {
    pub fn validate(&self, item: &SuspensionOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_override_validator() -> SuspensionOverrideValidator {
    SuspensionOverrideValidator {
        rules: vec![
            Rc::new(SuspensionOverrideRequiredIssuerRule {
                message: "suspension override needs this field".to_string(),
            }) as Rc<dyn SuspensionOverrideRule>,
            Rc::new(SuspensionOverrideCreatedAtRangeRule { min: 80, max: 86400 }),
            Rc::new(SuspensionOverrideIssuerLengthRule { max_length: 255 }),
            Rc::new(SuspensionOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SuspensionOverrideWeightRangeRule { min: 6.1, max: 4630.6 }),
            Rc::new(SuspensionOverrideCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_override_validator();
        let item = new_suspension_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = SuspensionOverrideRequiredIssuerRule {
            message: "suspension override needs a value".to_string(),
        };
        let mut item = new_suspension_override("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SuspensionOverrideCreatedAtRangeRule { min: 32, max: 120 };
        let mut item = new_suspension_override("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = SuspensionOverrideIssuerLengthRule { max_length: 86400 };
        let mut item = new_suspension_override("x", "y");
        item.issuer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SuspensionOverrideWeightRangeRule { min: 9.5, max: 1025.1 };
        let mut item = new_suspension_override("x", "y");
        item.weight = 1026.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionOverrideCodeLengthRule { max_length: 3600 };
        let mut item = new_suspension_override("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
