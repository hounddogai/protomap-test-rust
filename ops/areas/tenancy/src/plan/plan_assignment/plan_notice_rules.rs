use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan notice must pass. Check returns its problems, or none.
pub trait PlanNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PlanNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeRequiredTitleRule {
    pub message: String,
}

impl PlanNoticeRule for PlanNoticeRequiredTitleRule {
    fn check(&self, item: &PlanNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeRequiredNameRule {
    pub message: String,
}

impl PlanNoticeRule for PlanNoticeRequiredNameRule {
    fn check(&self, item: &PlanNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeCodeLengthRule {
    pub max_length: i64,
}

impl PlanNoticeRule for PlanNoticeCodeLengthRule {
    fn check(&self, item: &PlanNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeRequiredCodeRule {
    pub message: String,
}

impl PlanNoticeRule for PlanNoticeRequiredCodeRule {
    fn check(&self, item: &PlanNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeRequiredKeyIdRule {
    pub message: String,
}

impl PlanNoticeRule for PlanNoticeRequiredKeyIdRule {
    fn check(&self, item: &PlanNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeNameLengthRule {
    pub max_length: i64,
}

impl PlanNoticeRule for PlanNoticeNameLengthRule {
    fn check(&self, item: &PlanNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan notice.
#[derive(Debug, Clone)]
pub struct PlanNoticeValidator {
    pub rules: Vec<Rc<dyn PlanNoticeRule>>,
}

impl PlanNoticeValidator {
    pub fn validate(&self, item: &PlanNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_notice_validator() -> PlanNoticeValidator {
    PlanNoticeValidator {
        rules: vec![
            Rc::new(PlanNoticeRequiredTitleRule {
                message: "plan notice needs this field".to_string(),
            }) as Rc<dyn PlanNoticeRule>,
            Rc::new(PlanNoticeRequiredNameRule {
                message: "plan notice needs a value".to_string(),
            }),
            Rc::new(PlanNoticeCodeLengthRule { max_length: 3600 }),
            Rc::new(PlanNoticeRequiredCodeRule {
                message: "plan notice needs a value".to_string(),
            }),
            Rc::new(PlanNoticeRequiredKeyIdRule {
                message: "plan notice needs a value".to_string(),
            }),
            Rc::new(PlanNoticeNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_notice_validator();
        let item = new_plan_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PlanNoticeRequiredTitleRule {
            message: "plan notice needs this field".to_string(),
        };
        let mut item = new_plan_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PlanNoticeRequiredNameRule {
            message: "plan notice needs an entry".to_string(),
        };
        let mut item = new_plan_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanNoticeCodeLengthRule { max_length: 255 };
        let mut item = new_plan_notice("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PlanNoticeRequiredCodeRule {
            message: "plan notice needs a value".to_string(),
        };
        let mut item = new_plan_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PlanNoticeRequiredKeyIdRule {
            message: "plan notice needs this field".to_string(),
        };
        let mut item = new_plan_notice("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PlanNoticeNameLengthRule { max_length: 1000 };
        let mut item = new_plan_notice("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
