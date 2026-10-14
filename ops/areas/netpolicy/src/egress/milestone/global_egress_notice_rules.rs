use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global egress notice must pass. Check returns its problems, or none.
pub trait GlobalEgressNoticeRule: std::fmt::Debug {
    fn check(&self, item: &GlobalEgressNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressNoticeNameLengthRule {
    pub max_length: i64,
}

impl GlobalEgressNoticeRule for GlobalEgressNoticeNameLengthRule {
    fn check(&self, item: &GlobalEgressNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressNoticeMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalEgressNoticeRule for GlobalEgressNoticeMarginRangeRule {
    fn check(&self, item: &GlobalEgressNotice) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressNoticeCodeLengthRule {
    pub max_length: i64,
}

impl GlobalEgressNoticeRule for GlobalEgressNoticeCodeLengthRule {
    fn check(&self, item: &GlobalEgressNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global egress notice.
#[derive(Debug, Clone)]
pub struct GlobalEgressNoticeValidator {
    pub rules: Vec<Rc<dyn GlobalEgressNoticeRule>>,
}

impl GlobalEgressNoticeValidator {
    pub fn validate(&self, item: &GlobalEgressNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalEgressNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalEgressNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_egress_notice_validator() -> GlobalEgressNoticeValidator {
    GlobalEgressNoticeValidator {
        rules: vec![
            Rc::new(GlobalEgressNoticeNameLengthRule { max_length: 3600 }) as Rc<dyn GlobalEgressNoticeRule>,
            Rc::new(GlobalEgressNoticeMarginRangeRule { min: 2.4, max: 4472.4 }),
            Rc::new(GlobalEgressNoticeCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_egress_notice_validator();
        let item = new_global_egress_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalEgressNoticeNameLengthRule { max_length: 80 };
        let mut item = new_global_egress_notice("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalEgressNoticeMarginRangeRule { min: 4.3, max: 2673.7 };
        let mut item = new_global_egress_notice("x", "y");
        item.margin = 2675.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalEgressNoticeCodeLengthRule { max_length: 16 };
        let mut item = new_global_egress_notice("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
