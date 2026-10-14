use super::*;

use acme_kit::text;

/// Where a default timeline rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTimelineRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultTimelineRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultTimelineRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default timeline rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineRule {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub port: i64,
    pub version: i64,
    pub burst: i64,
    pub status: DefaultTimelineRuleStatus,
}

impl DefaultTimelineRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTimelineRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTimelineRuleStatus) -> bool {
        match self.status {
            DefaultTimelineRuleStatus::Created => {
                return next == DefaultTimelineRuleStatus::Validated;
            }
            DefaultTimelineRuleStatus::Validated => {
                return next == DefaultTimelineRuleStatus::Retired;
            }
            DefaultTimelineRuleStatus::Published => {
                match next {
                    DefaultTimelineRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultTimelineRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default timeline rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.version as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTimelineRule {
        DefaultTimelineRule {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            port: self.port,
            version: self.version,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default timeline rule in status created.
pub fn new_default_timeline_rule(code: &str, name: &str) -> DefaultTimelineRule {
    DefaultTimelineRule {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        port: 0,
        version: 0,
        burst: 0,
        status: DefaultTimelineRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_timeline_rule("code-339", "name-184");
        assert_eq!(item.status, DefaultTimelineRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_default_timeline_rule("code-339", "name-184");
        if item.move_to(DefaultTimelineRuleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTimelineRuleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_timeline_rule("code-339", "name-184");
        item.status = DefaultTimelineRuleStatus::Retired;
        assert!(
            item.move_to(DefaultTimelineRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
