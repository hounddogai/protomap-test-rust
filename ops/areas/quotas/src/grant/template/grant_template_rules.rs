use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant template must pass. Check returns its problems, or none.
pub trait GrantTemplateRule: std::fmt::Debug {
    fn check(&self, item: &GrantTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateActiveGuardRule {
    pub limit: i64,
}

impl GrantTemplateRule for GrantTemplateActiveGuardRule {
    fn check(&self, item: &GrantTemplate) -> Vec<String> {
        if item.active && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while active",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateRequiredCodeRule {
    pub message: String,
}

impl GrantTemplateRule for GrantTemplateRequiredCodeRule {
    fn check(&self, item: &GrantTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantTemplateRule for GrantTemplateExpiresAtRangeRule {
    fn check(&self, item: &GrantTemplate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateTimezoneLengthRule {
    pub max_length: i64,
}

impl GrantTemplateRule for GrantTemplateTimezoneLengthRule {
    fn check(&self, item: &GrantTemplate) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateRequiredNameRule {
    pub message: String,
}

impl GrantTemplateRule for GrantTemplateRequiredNameRule {
    fn check(&self, item: &GrantTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantTemplateRule for GrantTemplateWeightRangeRule {
    fn check(&self, item: &GrantTemplate) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant template.
#[derive(Debug, Clone)]
pub struct GrantTemplateValidator {
    pub rules: Vec<Rc<dyn GrantTemplateRule>>,
}

impl GrantTemplateValidator {
    pub fn validate(&self, item: &GrantTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_template_validator() -> GrantTemplateValidator {
    GrantTemplateValidator {
        rules: vec![
            Rc::new(GrantTemplateActiveGuardRule { limit: 64 }) as Rc<dyn GrantTemplateRule>,
            Rc::new(GrantTemplateRequiredCodeRule {
                message: "grant template needs a value".to_string(),
            }),
            Rc::new(GrantTemplateExpiresAtRangeRule { min: 32, max: 86400 }),
            Rc::new(GrantTemplateTimezoneLengthRule { max_length: 32 }),
            Rc::new(GrantTemplateRequiredNameRule {
                message: "grant template needs an entry".to_string(),
            }),
            Rc::new(GrantTemplateWeightRangeRule { min: 4.5, max: 4734.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_template_validator();
        let item = new_grant_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantTemplateRequiredCodeRule {
            message: "grant template needs a value".to_string(),
        };
        let mut item = new_grant_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = GrantTemplateExpiresAtRangeRule { min: 255, max: 86400 };
        let mut item = new_grant_template("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = GrantTemplateTimezoneLengthRule { max_length: 3600 };
        let mut item = new_grant_template("x", "y");
        item.timezone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = GrantTemplateRequiredNameRule {
            message: "grant template needs an entry".to_string(),
        };
        let mut item = new_grant_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = GrantTemplateWeightRangeRule { min: 7.6, max: 3166.0 };
        let mut item = new_grant_template("x", "y");
        item.weight = 3167.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
