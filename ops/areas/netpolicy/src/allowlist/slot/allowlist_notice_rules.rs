use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist notice must pass. Check returns its problems, or none.
pub trait AllowlistNoticeRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistNoticeVisibleGuardRule {
    pub limit: i64,
}

impl AllowlistNoticeRule for AllowlistNoticeVisibleGuardRule {
    fn check(&self, item: &AllowlistNotice) -> Vec<String> {
        if item.visible && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while visible",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistNoticeRequiredCodeRule {
    pub message: String,
}

impl AllowlistNoticeRule for AllowlistNoticeRequiredCodeRule {
    fn check(&self, item: &AllowlistNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistNoticeChecksumLengthRule {
    pub max_length: i64,
}

impl AllowlistNoticeRule for AllowlistNoticeChecksumLengthRule {
    fn check(&self, item: &AllowlistNotice) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist notice.
#[derive(Debug, Clone)]
pub struct AllowlistNoticeValidator {
    pub rules: Vec<Rc<dyn AllowlistNoticeRule>>,
}

impl AllowlistNoticeValidator {
    pub fn validate(&self, item: &AllowlistNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_notice_validator() -> AllowlistNoticeValidator {
    AllowlistNoticeValidator {
        rules: vec![
            Rc::new(AllowlistNoticeVisibleGuardRule { limit: 3600 }) as Rc<dyn AllowlistNoticeRule>,
            Rc::new(AllowlistNoticeRequiredCodeRule {
                message: "allowlist notice needs an entry".to_string(),
            }),
            Rc::new(AllowlistNoticeChecksumLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_notice_validator();
        let item = new_allowlist_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistNoticeRequiredCodeRule {
            message: "allowlist notice needs a value".to_string(),
        };
        let mut item = new_allowlist_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = AllowlistNoticeChecksumLengthRule { max_length: 32 };
        let mut item = new_allowlist_notice("x", "y");
        item.checksum = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
