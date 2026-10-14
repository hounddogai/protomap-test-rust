use super::*;

use acme_kit::text;

/// Where a rule schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RuleScheduleStatus {
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

impl std::fmt::Display for RuleScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSchedule {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub capacity: i64,
    pub segment: String,
    pub visible: bool,
    pub reviewer: String,
    pub currency: String,
    pub description: String,
    pub owner_id: String,
    pub status: RuleScheduleStatus,
}

impl RuleSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleScheduleStatus) -> bool {
        match self.status {
            RuleScheduleStatus::New => {
                match next {
                    RuleScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleScheduleStatus::Queued => {
                return next == RuleScheduleStatus::Done || next == RuleScheduleStatus::Failed;
            }
            RuleScheduleStatus::Running => {
                return next == RuleScheduleStatus::Done || next == RuleScheduleStatus::Failed;
            }
            RuleScheduleStatus::Done => {
                return next == RuleScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.capacity as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleSchedule {
        RuleSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            capacity: self.capacity,
            segment: self.segment.clone(),
            visible: self.visible,
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rule schedule in status new.
pub fn new_rule_schedule(code: &str, title: &str) -> RuleSchedule {
    RuleSchedule {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        capacity: 0,
        segment: "".to_string(),
        visible: false,
        reviewer: "".to_string(),
        currency: "".to_string(),
        description: "".to_string(),
        owner_id: "".to_string(),
        status: RuleScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rule_schedule("code-946", "title-726");
        assert_eq!(item.status, RuleScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rule_schedule("code-946", "title-726");
        if item.move_to(RuleScheduleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleScheduleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rule_schedule("code-946", "title-726");
        item.status = RuleScheduleStatus::Failed;
        assert!(item.move_to(RuleScheduleStatus::New).is_err(), "expected move to fail");
    }
}
