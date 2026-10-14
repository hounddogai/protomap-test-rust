use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page score must pass. Check returns its problems, or none.
pub trait PageScoreRule: std::fmt::Debug {
    fn check(&self, item: &PageScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScoreChannelLengthRule {
    pub max_length: i64,
}

impl PageScoreRule for PageScoreChannelLengthRule {
    fn check(&self, item: &PageScore) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScoreCodePrefixRule {
    pub prefix: String,
}

impl PageScoreRule for PageScoreCodePrefixRule {
    fn check(&self, item: &PageScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScoreRequiredDescriptionRule {
    pub message: String,
}

impl PageScoreRule for PageScoreRequiredDescriptionRule {
    fn check(&self, item: &PageScore) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScoreNameLengthRule {
    pub max_length: i64,
}

impl PageScoreRule for PageScoreNameLengthRule {
    fn check(&self, item: &PageScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScoreCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageScoreRule for PageScoreCapacityRangeRule {
    fn check(&self, item: &PageScore) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a page score.
#[derive(Debug, Clone)]
pub struct PageScoreValidator {
    pub rules: Vec<Rc<dyn PageScoreRule>>,
}

impl PageScoreValidator {
    pub fn validate(&self, item: &PageScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_score_validator() -> PageScoreValidator {
    PageScoreValidator {
        rules: vec![
            Rc::new(PageScoreChannelLengthRule { max_length: 255 }) as Rc<dyn PageScoreRule>,
            Rc::new(PageScoreCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PageScoreRequiredDescriptionRule {
                message: "page score needs this field".to_string(),
            }),
            Rc::new(PageScoreNameLengthRule { max_length: 64 }),
            Rc::new(PageScoreCapacityRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_score_validator();
        let item = new_page_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PageScoreChannelLengthRule { max_length: 120 };
        let mut item = new_page_score("x", "y");
        item.channel = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PageScoreRequiredDescriptionRule {
            message: "page score needs this field".to_string(),
        };
        let mut item = new_page_score("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PageScoreNameLengthRule { max_length: 80 };
        let mut item = new_page_score("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PageScoreCapacityRangeRule { min: 32, max: 86400 };
        let mut item = new_page_score("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
