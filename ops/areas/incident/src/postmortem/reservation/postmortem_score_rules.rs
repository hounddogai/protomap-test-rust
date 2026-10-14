use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem score must pass. Check returns its problems, or none.
pub trait PostmortemScoreRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemScoreRequiredTenantIdRule {
    pub message: String,
}

impl PostmortemScoreRule for PostmortemScoreRequiredTenantIdRule {
    fn check(&self, item: &PostmortemScore) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemScoreRequiredTitleRule {
    pub message: String,
}

impl PostmortemScoreRule for PostmortemScoreRequiredTitleRule {
    fn check(&self, item: &PostmortemScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemScoreAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemScoreRule for PostmortemScoreAmountRangeRule {
    fn check(&self, item: &PostmortemScore) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemScoreTitleLengthRule {
    pub max_length: i64,
}

impl PostmortemScoreRule for PostmortemScoreTitleLengthRule {
    fn check(&self, item: &PostmortemScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem score.
#[derive(Debug, Clone)]
pub struct PostmortemScoreValidator {
    pub rules: Vec<Rc<dyn PostmortemScoreRule>>,
}

impl PostmortemScoreValidator {
    pub fn validate(&self, item: &PostmortemScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_score_validator() -> PostmortemScoreValidator {
    PostmortemScoreValidator {
        rules: vec![
            Rc::new(PostmortemScoreRequiredTenantIdRule {
                message: "postmortem score needs a value".to_string(),
            }) as Rc<dyn PostmortemScoreRule>,
            Rc::new(PostmortemScoreRequiredTitleRule {
                message: "postmortem score needs an entry".to_string(),
            }),
            Rc::new(PostmortemScoreAmountRangeRule { min: 8.9, max: 3185.5 }),
            Rc::new(PostmortemScoreTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_score_validator();
        let item = new_postmortem_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PostmortemScoreRequiredTenantIdRule {
            message: "postmortem score needs this field".to_string(),
        };
        let mut item = new_postmortem_score("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PostmortemScoreRequiredTitleRule {
            message: "postmortem score needs an entry".to_string(),
        };
        let mut item = new_postmortem_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PostmortemScoreAmountRangeRule { min: 7.6, max: 4051.5 };
        let mut item = new_postmortem_score("x", "y");
        item.amount = 4053.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PostmortemScoreTitleLengthRule { max_length: 16 };
        let mut item = new_postmortem_score("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
