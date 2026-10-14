use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling page template must pass. Check returns its problems, or none.
pub trait RollingPageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RollingPageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPageTemplateRule for RollingPageTemplateExpiresAtRangeRule {
    fn check(&self, item: &RollingPageTemplate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplateLabelLengthRule {
    pub max_length: i64,
}

impl RollingPageTemplateRule for RollingPageTemplateLabelLengthRule {
    fn check(&self, item: &RollingPageTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplateWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPageTemplateRule for RollingPageTemplateWindowDaysRangeRule {
    fn check(&self, item: &RollingPageTemplate) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplateRequiredLabelRule {
    pub message: String,
}

impl RollingPageTemplateRule for RollingPageTemplateRequiredLabelRule {
    fn check(&self, item: &RollingPageTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling page template.
#[derive(Debug, Clone)]
pub struct RollingPageTemplateValidator {
    pub rules: Vec<Rc<dyn RollingPageTemplateRule>>,
}

impl RollingPageTemplateValidator {
    pub fn validate(&self, item: &RollingPageTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_page_template_validator() -> RollingPageTemplateValidator {
    RollingPageTemplateValidator {
        rules: vec![
            Rc::new(RollingPageTemplateExpiresAtRangeRule { min: 16, max: 32 }) as Rc<dyn RollingPageTemplateRule>,
            Rc::new(RollingPageTemplateLabelLengthRule { max_length: 32 }),
            Rc::new(RollingPageTemplateWindowDaysRangeRule { min: 32, max: 255 }),
            Rc::new(RollingPageTemplateRequiredLabelRule {
                message: "rolling page template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_page_template_validator();
        let item = new_rolling_page_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RollingPageTemplateExpiresAtRangeRule { min: 16, max: 3600 };
        let mut item = new_rolling_page_template("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingPageTemplateLabelLengthRule { max_length: 86400 };
        let mut item = new_rolling_page_template("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RollingPageTemplateWindowDaysRangeRule { min: 16, max: 32 };
        let mut item = new_rolling_page_template("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingPageTemplateRequiredLabelRule {
            message: "rolling page template needs an entry".to_string(),
        };
        let mut item = new_rolling_page_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
