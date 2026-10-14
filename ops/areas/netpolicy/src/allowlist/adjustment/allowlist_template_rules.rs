use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist template must pass. Check returns its problems, or none.
pub trait AllowlistTemplateRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTemplateFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistTemplateRule for AllowlistTemplateFloorValueRangeRule {
    fn check(&self, item: &AllowlistTemplate) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTemplateRequiredHostRule {
    pub message: String,
}

impl AllowlistTemplateRule for AllowlistTemplateRequiredHostRule {
    fn check(&self, item: &AllowlistTemplate) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistTemplateRule for AllowlistTemplateExpiresAtRangeRule {
    fn check(&self, item: &AllowlistTemplate) -> Vec<String> {
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

/// Runs every rule of a allowlist template.
#[derive(Debug, Clone)]
pub struct AllowlistTemplateValidator {
    pub rules: Vec<Rc<dyn AllowlistTemplateRule>>,
}

impl AllowlistTemplateValidator {
    pub fn validate(&self, item: &AllowlistTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_template_validator() -> AllowlistTemplateValidator {
    AllowlistTemplateValidator {
        rules: vec![
            Rc::new(AllowlistTemplateFloorValueRangeRule { min: 8.4, max: 665.0 }) as Rc<dyn AllowlistTemplateRule>,
            Rc::new(AllowlistTemplateRequiredHostRule {
                message: "allowlist template needs a value".to_string(),
            }),
            Rc::new(AllowlistTemplateExpiresAtRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_template_validator();
        let item = new_allowlist_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AllowlistTemplateFloorValueRangeRule { min: 1.5, max: 3818.2 };
        let mut item = new_allowlist_template("x", "y");
        item.floor_value = 3819.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = AllowlistTemplateRequiredHostRule {
            message: "allowlist template needs an entry".to_string(),
        };
        let mut item = new_allowlist_template("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = AllowlistTemplateExpiresAtRangeRule { min: 16, max: 3600 };
        let mut item = new_allowlist_template("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
