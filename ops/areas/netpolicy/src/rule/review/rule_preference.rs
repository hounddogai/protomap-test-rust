use super::*;

use acme_kit::text;

/// Where a rule preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RulePreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RulePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RulePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RulePreference {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub region: String,
    pub cost: f64,
    pub status: RulePreferenceStatus,
}

impl RulePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RulePreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RulePreferenceStatus) -> bool {
        match self.status {
            RulePreferenceStatus::New => {
                return next == RulePreferenceStatus::Queued || next == RulePreferenceStatus::Done;
            }
            RulePreferenceStatus::Queued => {
                return next == RulePreferenceStatus::Done || next == RulePreferenceStatus::Failed;
            }
            RulePreferenceStatus::Running => {
                return next == RulePreferenceStatus::Done || next == RulePreferenceStatus::Failed;
            }
            RulePreferenceStatus::Done => {
                return next == RulePreferenceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RulePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule preference", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> RulePreference {
        RulePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            region: self.region.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rule preference in status new.
pub fn new_rule_preference(code: &str, title: &str) -> RulePreference {
    RulePreference {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        region: "".to_string(),
        cost: 0.0,
        status: RulePreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rule_preference("code-81", "title-813");
        assert_eq!(item.status, RulePreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rule_preference("code-81", "title-813");
        if item.move_to(RulePreferenceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RulePreferenceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rule_preference("code-81", "title-813");
        item.status = RulePreferenceStatus::Failed;
        assert!(
            item.move_to(RulePreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
