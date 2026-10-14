use super::*;

use acme_kit::text;

/// Where a limit rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LimitRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LimitRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRule {
    pub code: String,
    pub title: String,
    pub country: String,
    pub revision: i64,
    pub maximum: f64,
    pub key_id: String,
    pub pinned: bool,
    pub checksum: String,
    pub status: LimitRuleStatus,
}

impl LimitRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitRuleStatus) -> bool {
        match self.status {
            LimitRuleStatus::Planned => {
                match next {
                    LimitRuleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRuleStatus::Scheduled => {
                match next {
                    LimitRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRuleStatus::InProgress => {
                match next {
                    LimitRuleStatus::Completed | LimitRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRuleStatus::Completed => {
                match next {
                    LimitRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LimitRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.revision as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitRule {
        LimitRule {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            revision: self.revision,
            maximum: self.maximum,
            key_id: self.key_id.clone(),
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a limit rule in status planned.
pub fn new_limit_rule(code: &str, title: &str) -> LimitRule {
    LimitRule {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        revision: 0,
        maximum: 0.0,
        key_id: "".to_string(),
        pinned: false,
        checksum: "".to_string(),
        status: LimitRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_limit_rule("code-175", "title-493");
        assert_eq!(item.status, LimitRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_limit_rule("code-175", "title-493");
        if item.move_to(LimitRuleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitRuleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_limit_rule("code-175", "title-493");
        item.status = LimitRuleStatus::Cancelled;
        assert!(item.move_to(LimitRuleStatus::Planned).is_err(), "expected move to fail");
    }
}
