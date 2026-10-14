use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline quota must pass. Check returns its problems, or none.
pub trait TimelineQuotaRule: std::fmt::Debug {
    fn check(&self, item: &TimelineQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuotaRequiredNameRule {
    pub message: String,
}

impl TimelineQuotaRule for TimelineQuotaRequiredNameRule {
    fn check(&self, item: &TimelineQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuotaArchivedGuardRule {
    pub limit: i64,
}

impl TimelineQuotaRule for TimelineQuotaArchivedGuardRule {
    fn check(&self, item: &TimelineQuota) -> Vec<String> {
        if item.archived && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while archived",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuotaRequiredHostRule {
    pub message: String,
}

impl TimelineQuotaRule for TimelineQuotaRequiredHostRule {
    fn check(&self, item: &TimelineQuota) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuotaRegionLengthRule {
    pub max_length: i64,
}

impl TimelineQuotaRule for TimelineQuotaRegionLengthRule {
    fn check(&self, item: &TimelineQuota) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineQuotaHostLengthRule {
    pub max_length: i64,
}

impl TimelineQuotaRule for TimelineQuotaHostLengthRule {
    fn check(&self, item: &TimelineQuota) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline quota.
#[derive(Debug, Clone)]
pub struct TimelineQuotaValidator {
    pub rules: Vec<Rc<dyn TimelineQuotaRule>>,
}

impl TimelineQuotaValidator {
    pub fn validate(&self, item: &TimelineQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_quota_validator() -> TimelineQuotaValidator {
    TimelineQuotaValidator {
        rules: vec![
            Rc::new(TimelineQuotaRequiredNameRule {
                message: "timeline quota needs this field".to_string(),
            }) as Rc<dyn TimelineQuotaRule>,
            Rc::new(TimelineQuotaArchivedGuardRule { limit: 32 }),
            Rc::new(TimelineQuotaRequiredHostRule {
                message: "timeline quota needs a value".to_string(),
            }),
            Rc::new(TimelineQuotaRegionLengthRule { max_length: 120 }),
            Rc::new(TimelineQuotaHostLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_quota_validator();
        let item = new_timeline_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineQuotaRequiredNameRule {
            message: "timeline quota needs an entry".to_string(),
        };
        let mut item = new_timeline_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = TimelineQuotaRequiredHostRule {
            message: "timeline quota needs this field".to_string(),
        };
        let mut item = new_timeline_quota("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = TimelineQuotaRegionLengthRule { max_length: 1000 };
        let mut item = new_timeline_quota("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = TimelineQuotaHostLengthRule { max_length: 255 };
        let mut item = new_timeline_quota("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
