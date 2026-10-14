use super::*;

use acme_kit::text;

/// Where a rule template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RuleTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RuleTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleTemplate {
    pub code: String,
    pub title: String,
    pub description: String,
    pub maximum: f64,
    pub category: String,
    pub scope: String,
    pub cost: f64,
    pub status: RuleTemplateStatus,
}

impl RuleTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleTemplateStatus) -> bool {
        match self.status {
            RuleTemplateStatus::Idle => {
                return next == RuleTemplateStatus::Draining;
            }
            RuleTemplateStatus::Warming => {
                return next == RuleTemplateStatus::Ready;
            }
            RuleTemplateStatus::Ready => {
                return next == RuleTemplateStatus::Draining || next == RuleTemplateStatus::Stopped;
            }
            RuleTemplateStatus::Draining => {
                match next {
                    RuleTemplateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RuleTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule template", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleTemplate {
        RuleTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            maximum: self.maximum,
            category: self.category.clone(),
            scope: self.scope.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rule template in status idle.
pub fn new_rule_template(code: &str, title: &str) -> RuleTemplate {
    RuleTemplate {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        maximum: 0.0,
        category: "".to_string(),
        scope: "".to_string(),
        cost: 0.0,
        status: RuleTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rule_template("code-222", "title-850");
        assert_eq!(item.status, RuleTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_rule_template("code-222", "title-850");
        if item.move_to(RuleTemplateStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleTemplateStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rule_template("code-222", "title-850");
        item.status = RuleTemplateStatus::Stopped;
        assert!(item.move_to(RuleTemplateStatus::Idle).is_err(), "expected move to fail");
    }
}
