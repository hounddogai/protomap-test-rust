use super::*;

use acme_kit::text;

/// Where a scheduled zone score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledZoneScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledZoneScoreStatus {
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

impl std::fmt::Display for ScheduledZoneScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled zone score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScore {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub segment: String,
    pub cost: f64,
    pub status: ScheduledZoneScoreStatus,
}

impl ScheduledZoneScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledZoneScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledZoneScoreStatus) -> bool {
        match self.status {
            ScheduledZoneScoreStatus::New => {
                return next == ScheduledZoneScoreStatus::Done;
            }
            ScheduledZoneScoreStatus::Queued => {
                return next == ScheduledZoneScoreStatus::Done;
            }
            ScheduledZoneScoreStatus::Running => {
                return next == ScheduledZoneScoreStatus::Done || next == ScheduledZoneScoreStatus::Failed;
            }
            ScheduledZoneScoreStatus::Done => {
                match next {
                    ScheduledZoneScoreStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledZoneScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled zone score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.weight + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledZoneScore {
        ScheduledZoneScore {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            segment: self.segment.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled zone score in status new.
pub fn new_scheduled_zone_score(code: &str, name: &str) -> ScheduledZoneScore {
    ScheduledZoneScore {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        segment: "".to_string(),
        cost: 0.0,
        status: ScheduledZoneScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_zone_score("code-894", "name-21");
        assert_eq!(item.status, ScheduledZoneScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_scheduled_zone_score("code-894", "name-21");
        if item.move_to(ScheduledZoneScoreStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledZoneScoreStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_zone_score("code-894", "name-21");
        item.status = ScheduledZoneScoreStatus::Failed;
        assert!(
            item.move_to(ScheduledZoneScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
