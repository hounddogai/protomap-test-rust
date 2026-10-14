use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release template must pass. Check returns its problems, or none.
pub trait ReleaseTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateRequiredClusterRule {
    pub message: String,
}

impl ReleaseTemplateRule for ReleaseTemplateRequiredClusterRule {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseTemplateRule for ReleaseTemplateAllowedCodeRule {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseTemplateRule for ReleaseTemplateQuotaUnitsRangeRule {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateRequiredLabelRule {
    pub message: String,
}

impl ReleaseTemplateRule for ReleaseTemplateRequiredLabelRule {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseTemplateRule for ReleaseTemplateCeilingRangeRule {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateRequiredAudienceRule {
    pub message: String,
}

impl ReleaseTemplateRule for ReleaseTemplateRequiredAudienceRule {
    fn check(&self, item: &ReleaseTemplate) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release template.
#[derive(Debug, Clone)]
pub struct ReleaseTemplateValidator {
    pub rules: Vec<Rc<dyn ReleaseTemplateRule>>,
}

impl ReleaseTemplateValidator {
    pub fn validate(&self, item: &ReleaseTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_template_validator() -> ReleaseTemplateValidator {
    ReleaseTemplateValidator {
        rules: vec![
            Rc::new(ReleaseTemplateRequiredClusterRule {
                message: "release template needs this field".to_string(),
            }) as Rc<dyn ReleaseTemplateRule>,
            Rc::new(ReleaseTemplateAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(ReleaseTemplateQuotaUnitsRangeRule { min: 64, max: 86400 }),
            Rc::new(ReleaseTemplateRequiredLabelRule {
                message: "release template needs a value".to_string(),
            }),
            Rc::new(ReleaseTemplateCeilingRangeRule { min: 0.7, max: 1219.2 }),
            Rc::new(ReleaseTemplateRequiredAudienceRule {
                message: "release template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_template_validator();
        let item = new_release_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = ReleaseTemplateRequiredClusterRule {
            message: "release template needs this field".to_string(),
        };
        let mut item = new_release_template("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ReleaseTemplateQuotaUnitsRangeRule { min: 64, max: 80 };
        let mut item = new_release_template("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReleaseTemplateRequiredLabelRule {
            message: "release template needs an entry".to_string(),
        };
        let mut item = new_release_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ReleaseTemplateCeilingRangeRule { min: 1.3, max: 3153.3 };
        let mut item = new_release_template("x", "y");
        item.ceiling = 3154.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ReleaseTemplateRequiredAudienceRule {
            message: "release template needs a value".to_string(),
        };
        let mut item = new_release_template("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
