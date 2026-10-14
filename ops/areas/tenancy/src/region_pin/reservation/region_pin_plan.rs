use super::*;

use acme_kit::text;

/// Where a region pin plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionPinPlanStatus {
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

impl std::fmt::Display for RegionPinPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPlan {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub quota_units: i64,
    pub active: bool,
    pub cipher: String,
    pub margin: f64,
    pub reviewer: String,
    pub status: RegionPinPlanStatus,
}

impl RegionPinPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinPlanStatus) -> bool {
        match self.status {
            RegionPinPlanStatus::New => {
                match next {
                    RegionPinPlanStatus::Queued | RegionPinPlanStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinPlanStatus::Queued => {
                match next {
                    RegionPinPlanStatus::Running | RegionPinPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinPlanStatus::Running => {
                match next {
                    RegionPinPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinPlanStatus::Done => {
                match next {
                    RegionPinPlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionPinPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinPlan {
        RegionPinPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            quota_units: self.quota_units,
            active: self.active,
            cipher: self.cipher.clone(),
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a region pin plan in status new.
pub fn new_region_pin_plan(code: &str, name: &str) -> RegionPinPlan {
    RegionPinPlan {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        quota_units: 0,
        active: false,
        cipher: "".to_string(),
        margin: 0.0,
        reviewer: "".to_string(),
        status: RegionPinPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_region_pin_plan("code-861", "name-696");
        assert_eq!(item.status, RegionPinPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_region_pin_plan("code-861", "name-696");
        if item.move_to(RegionPinPlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinPlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_region_pin_plan("code-861", "name-696");
        item.status = RegionPinPlanStatus::Failed;
        assert!(item.move_to(RegionPinPlanStatus::New).is_err(), "expected move to fail");
    }
}
