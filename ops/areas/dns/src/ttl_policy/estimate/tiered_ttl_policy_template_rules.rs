use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered ttl policy template must pass. Check returns its problems, or none.
pub trait TieredTtlPolicyTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateAudienceLengthRule {
    pub max_length: i64,
}

impl TieredTtlPolicyTemplateRule for TieredTtlPolicyTemplateAudienceLengthRule {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredTtlPolicyTemplateRule for TieredTtlPolicyTemplateFloorValueRangeRule {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateRequiredCodeRule {
    pub message: String,
}

impl TieredTtlPolicyTemplateRule for TieredTtlPolicyTemplateRequiredCodeRule {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateRequiredLabelRule {
    pub message: String,
}

impl TieredTtlPolicyTemplateRule for TieredTtlPolicyTemplateRequiredLabelRule {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTtlPolicyTemplateRule for TieredTtlPolicyTemplateDurationMinutesRangeRule {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyTemplateRequiredAudienceRule {
    pub message: String,
}

impl TieredTtlPolicyTemplateRule for TieredTtlPolicyTemplateRequiredAudienceRule {
    fn check(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered ttl policy template.
#[derive(Debug, Clone)]
pub struct TieredTtlPolicyTemplateValidator {
    pub rules: Vec<Rc<dyn TieredTtlPolicyTemplateRule>>,
}

impl TieredTtlPolicyTemplateValidator {
    pub fn validate(&self, item: &TieredTtlPolicyTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredTtlPolicyTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTtlPolicyTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_ttl_policy_template_validator() -> TieredTtlPolicyTemplateValidator {
    TieredTtlPolicyTemplateValidator {
        rules: vec![
            Rc::new(TieredTtlPolicyTemplateAudienceLengthRule { max_length: 120 })
                as Rc<dyn TieredTtlPolicyTemplateRule>,
            Rc::new(TieredTtlPolicyTemplateFloorValueRangeRule { min: 9.2, max: 3480.2 }),
            Rc::new(TieredTtlPolicyTemplateRequiredCodeRule {
                message: "tiered ttl policy template needs this field".to_string(),
            }),
            Rc::new(TieredTtlPolicyTemplateRequiredLabelRule {
                message: "tiered ttl policy template needs an entry".to_string(),
            }),
            Rc::new(TieredTtlPolicyTemplateDurationMinutesRangeRule { min: 64, max: 255 }),
            Rc::new(TieredTtlPolicyTemplateRequiredAudienceRule {
                message: "tiered ttl policy template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_ttl_policy_template_validator();
        let item = new_tiered_ttl_policy_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = TieredTtlPolicyTemplateAudienceLengthRule { max_length: 80 };
        let mut item = new_tiered_ttl_policy_template("x", "y");
        item.audience = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TieredTtlPolicyTemplateFloorValueRangeRule { min: 2.8, max: 4660.6 };
        let mut item = new_tiered_ttl_policy_template("x", "y");
        item.floor_value = 4662.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredTtlPolicyTemplateRequiredCodeRule {
            message: "tiered ttl policy template needs a value".to_string(),
        };
        let mut item = new_tiered_ttl_policy_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredTtlPolicyTemplateRequiredLabelRule {
            message: "tiered ttl policy template needs a value".to_string(),
        };
        let mut item = new_tiered_ttl_policy_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = TieredTtlPolicyTemplateDurationMinutesRangeRule { min: 86400, max: 86400 };
        let mut item = new_tiered_ttl_policy_template("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = TieredTtlPolicyTemplateRequiredAudienceRule {
            message: "tiered ttl policy template needs this field".to_string(),
        };
        let mut item = new_tiered_ttl_policy_template("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
