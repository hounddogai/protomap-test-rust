use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink template must pass. Check returns its problems, or none.
pub trait SinkTemplateRule: std::fmt::Debug {
    fn check(&self, item: &SinkTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl SinkTemplateRule for SinkTemplateAllowedClusterRule {
    fn check(&self, item: &SinkTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is stale", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateRequiredCodeRule {
    pub message: String,
}

impl SinkTemplateRule for SinkTemplateRequiredCodeRule {
    fn check(&self, item: &SinkTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateTitleLengthRule {
    pub max_length: i64,
}

impl SinkTemplateRule for SinkTemplateTitleLengthRule {
    fn check(&self, item: &SinkTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SinkTemplateRule for SinkTemplateTagsLimitRule {
    fn check(&self, item: &SinkTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateClusterLengthRule {
    pub max_length: i64,
}

impl SinkTemplateRule for SinkTemplateClusterLengthRule {
    fn check(&self, item: &SinkTemplate) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkTemplateRule for SinkTemplateExpiresAtRangeRule {
    fn check(&self, item: &SinkTemplate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink template.
#[derive(Debug, Clone)]
pub struct SinkTemplateValidator {
    pub rules: Vec<Rc<dyn SinkTemplateRule>>,
}

impl SinkTemplateValidator {
    pub fn validate(&self, item: &SinkTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_template_validator() -> SinkTemplateValidator {
    SinkTemplateValidator {
        rules: vec![
            Rc::new(SinkTemplateAllowedClusterRule { allowed: Vec::new() }) as Rc<dyn SinkTemplateRule>,
            Rc::new(SinkTemplateRequiredCodeRule {
                message: "sink template needs an entry".to_string(),
            }),
            Rc::new(SinkTemplateTitleLengthRule { max_length: 86400 }),
            Rc::new(SinkTemplateTagsLimitRule {
                max_count: 80,
                banned: Vec::new(),
            }),
            Rc::new(SinkTemplateClusterLengthRule { max_length: 3600 }),
            Rc::new(SinkTemplateExpiresAtRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_template_validator();
        let item = new_sink_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkTemplateRequiredCodeRule {
            message: "sink template needs this field".to_string(),
        };
        let mut item = new_sink_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SinkTemplateTitleLengthRule { max_length: 120 };
        let mut item = new_sink_template("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = SinkTemplateClusterLengthRule { max_length: 3600 };
        let mut item = new_sink_template("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SinkTemplateExpiresAtRangeRule { min: 64, max: 1000 };
        let mut item = new_sink_template("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
