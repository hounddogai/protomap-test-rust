use super::*;

use acme_kit::text;

/// Where a region pin milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionPinMilestoneStatus {
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

impl std::fmt::Display for RegionPinMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestone {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub name: String,
    pub rotation_days: i64,
    pub limit: i64,
    pub currency: String,
    pub updated_at: i64,
    pub created_at: i64,
    pub title: String,
    pub status: RegionPinMilestoneStatus,
}

impl RegionPinMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinMilestoneStatus) -> bool {
        match self.status {
            RegionPinMilestoneStatus::New => {
                match next {
                    RegionPinMilestoneStatus::Running | RegionPinMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinMilestoneStatus::Queued => {
                return next == RegionPinMilestoneStatus::Running || next == RegionPinMilestoneStatus::Done;
            }
            RegionPinMilestoneStatus::Running => {
                return next == RegionPinMilestoneStatus::Done || next == RegionPinMilestoneStatus::Failed;
            }
            RegionPinMilestoneStatus::Done => {
                return next == RegionPinMilestoneStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ratio - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinMilestone {
        RegionPinMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            limit: self.limit,
            currency: self.currency.clone(),
            updated_at: self.updated_at,
            created_at: self.created_at,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin milestone in status new.
pub fn new_region_pin_milestone(code: &str, label: &str) -> RegionPinMilestone {
    RegionPinMilestone {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        name: "".to_string(),
        rotation_days: 0,
        limit: 0,
        currency: "".to_string(),
        updated_at: 0,
        created_at: 0,
        title: "".to_string(),
        status: RegionPinMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_region_pin_milestone("code-816", "label-29");
        assert_eq!(item.status, RegionPinMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_region_pin_milestone("code-816", "label-29");
        if item.move_to(RegionPinMilestoneStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinMilestoneStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_region_pin_milestone("code-816", "label-29");
        item.status = RegionPinMilestoneStatus::Failed;
        assert!(
            item.move_to(RegionPinMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
