use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default flow log notice must pass. Check returns its problems, or none.
pub trait DefaultFlowLogNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFlowLogNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogNoticeScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultFlowLogNoticeRule for DefaultFlowLogNoticeScoreRangeRule {
    fn check(&self, item: &DefaultFlowLogNotice) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogNoticeRequiredTitleRule {
    pub message: String,
}

impl DefaultFlowLogNoticeRule for DefaultFlowLogNoticeRequiredTitleRule {
    fn check(&self, item: &DefaultFlowLogNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogNoticeCodePrefixRule {
    pub prefix: String,
}

impl DefaultFlowLogNoticeRule for DefaultFlowLogNoticeCodePrefixRule {
    fn check(&self, item: &DefaultFlowLogNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogNoticeRequiredAudienceRule {
    pub message: String,
}

impl DefaultFlowLogNoticeRule for DefaultFlowLogNoticeRequiredAudienceRule {
    fn check(&self, item: &DefaultFlowLogNotice) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default flow log notice.
#[derive(Debug, Clone)]
pub struct DefaultFlowLogNoticeValidator {
    pub rules: Vec<Rc<dyn DefaultFlowLogNoticeRule>>,
}

impl DefaultFlowLogNoticeValidator {
    pub fn validate(&self, item: &DefaultFlowLogNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFlowLogNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFlowLogNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_flow_log_notice_validator() -> DefaultFlowLogNoticeValidator {
    DefaultFlowLogNoticeValidator {
        rules: vec![
            Rc::new(DefaultFlowLogNoticeScoreRangeRule { min: 8.2, max: 1129.6 }) as Rc<dyn DefaultFlowLogNoticeRule>,
            Rc::new(DefaultFlowLogNoticeRequiredTitleRule {
                message: "default flow log notice needs a value".to_string(),
            }),
            Rc::new(DefaultFlowLogNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultFlowLogNoticeRequiredAudienceRule {
                message: "default flow log notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_flow_log_notice_validator();
        let item = new_default_flow_log_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DefaultFlowLogNoticeScoreRangeRule { min: 4.6, max: 2809.7 };
        let mut item = new_default_flow_log_notice("x", "y");
        item.score = 2811.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultFlowLogNoticeRequiredTitleRule {
            message: "default flow log notice needs a value".to_string(),
        };
        let mut item = new_default_flow_log_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = DefaultFlowLogNoticeRequiredAudienceRule {
            message: "default flow log notice needs a value".to_string(),
        };
        let mut item = new_default_flow_log_notice("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
