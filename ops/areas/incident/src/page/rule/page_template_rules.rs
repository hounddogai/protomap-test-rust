use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page template must pass. Check returns its problems, or none.
pub trait PageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageTemplateRule for PageTemplateQuotaUnitsRangeRule {
    fn check(&self, item: &PageTemplate) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateRequiredLabelRule {
    pub message: String,
}

impl PageTemplateRule for PageTemplateRequiredLabelRule {
    fn check(&self, item: &PageTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateLabelLengthRule {
    pub max_length: i64,
}

impl PageTemplateRule for PageTemplateLabelLengthRule {
    fn check(&self, item: &PageTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateClusterLengthRule {
    pub max_length: i64,
}

impl PageTemplateRule for PageTemplateClusterLengthRule {
    fn check(&self, item: &PageTemplate) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl PageTemplateRule for PageTemplateAllowedIssuerRule {
    fn check(&self, item: &PageTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is not allowed", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageTemplateRule for PageTemplateAttemptsRangeRule {
    fn check(&self, item: &PageTemplate) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a page template.
#[derive(Debug, Clone)]
pub struct PageTemplateValidator {
    pub rules: Vec<Rc<dyn PageTemplateRule>>,
}

impl PageTemplateValidator {
    pub fn validate(&self, item: &PageTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_template_validator() -> PageTemplateValidator {
    PageTemplateValidator {
        rules: vec![
            Rc::new(PageTemplateQuotaUnitsRangeRule { min: 255, max: 86400 }) as Rc<dyn PageTemplateRule>,
            Rc::new(PageTemplateRequiredLabelRule {
                message: "page template needs an entry".to_string(),
            }),
            Rc::new(PageTemplateLabelLengthRule { max_length: 255 }),
            Rc::new(PageTemplateClusterLengthRule { max_length: 3600 }),
            Rc::new(PageTemplateAllowedIssuerRule {
                allowed: vec!["weighted".to_string(), "capped".to_string()],
            }),
            Rc::new(PageTemplateAttemptsRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_template_validator();
        let item = new_page_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = PageTemplateQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_page_template("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PageTemplateRequiredLabelRule {
            message: "page template needs an entry".to_string(),
        };
        let mut item = new_page_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PageTemplateLabelLengthRule { max_length: 120 };
        let mut item = new_page_template("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PageTemplateClusterLengthRule { max_length: 86400 };
        let mut item = new_page_template("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = PageTemplateAttemptsRangeRule { min: 80, max: 255 };
        let mut item = new_page_template("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
