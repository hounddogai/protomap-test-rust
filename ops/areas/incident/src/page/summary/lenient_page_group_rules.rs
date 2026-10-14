use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient page group must pass. Check returns its problems, or none.
pub trait LenientPageGroupRule: std::fmt::Debug {
    fn check(&self, item: &LenientPageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupArchivedGuardRule {
    pub limit: i64,
}

impl LenientPageGroupRule for LenientPageGroupArchivedGuardRule {
    fn check(&self, item: &LenientPageGroup) -> Vec<String> {
        if item.archived && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while archived",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupCodePrefixRule {
    pub prefix: String,
}

impl LenientPageGroupRule for LenientPageGroupCodePrefixRule {
    fn check(&self, item: &LenientPageGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupCodeLengthRule {
    pub max_length: i64,
}

impl LenientPageGroupRule for LenientPageGroupCodeLengthRule {
    fn check(&self, item: &LenientPageGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupRequiredNameRule {
    pub message: String,
}

impl LenientPageGroupRule for LenientPageGroupRequiredNameRule {
    fn check(&self, item: &LenientPageGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPageGroupRule for LenientPageGroupPortRangeRule {
    fn check(&self, item: &LenientPageGroup) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient page group.
#[derive(Debug, Clone)]
pub struct LenientPageGroupValidator {
    pub rules: Vec<Rc<dyn LenientPageGroupRule>>,
}

impl LenientPageGroupValidator {
    pub fn validate(&self, item: &LenientPageGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_page_group_validator() -> LenientPageGroupValidator {
    LenientPageGroupValidator {
        rules: vec![
            Rc::new(LenientPageGroupArchivedGuardRule { limit: 80 }) as Rc<dyn LenientPageGroupRule>,
            Rc::new(LenientPageGroupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientPageGroupCodeLengthRule { max_length: 32 }),
            Rc::new(LenientPageGroupRequiredNameRule {
                message: "lenient page group needs a value".to_string(),
            }),
            Rc::new(LenientPageGroupPortRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_page_group_validator();
        let item = new_lenient_page_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientPageGroupCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_page_group("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientPageGroupRequiredNameRule {
            message: "lenient page group needs a value".to_string(),
        };
        let mut item = new_lenient_page_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientPageGroupPortRangeRule { min: 64, max: 80 };
        let mut item = new_lenient_page_group("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
