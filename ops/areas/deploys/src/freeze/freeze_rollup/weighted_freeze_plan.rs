use super::*;

use acme_kit::text;

/// Where a weighted freeze plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFreezePlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedFreezePlanStatus {
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

impl std::fmt::Display for WeightedFreezePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted freeze plan as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFreezePlan {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub budget: f64,
    pub weight: f64,
    pub phone: String,
    pub status: WeightedFreezePlanStatus,
}

impl WeightedFreezePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFreezePlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFreezePlanStatus) -> bool {
        match self.status {
            WeightedFreezePlanStatus::New => {
                return next == WeightedFreezePlanStatus::Queued || next == WeightedFreezePlanStatus::Failed;
            }
            WeightedFreezePlanStatus::Queued => {
                match next {
                    WeightedFreezePlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFreezePlanStatus::Running => {
                return next == WeightedFreezePlanStatus::Failed;
            }
            WeightedFreezePlanStatus::Done => {
                match next {
                    WeightedFreezePlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedFreezePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted freeze plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.budget - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFreezePlan {
        WeightedFreezePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            budget: self.budget,
            weight: self.weight,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted freeze plan in status new.
pub fn new_weighted_freeze_plan(code: &str, name: &str) -> WeightedFreezePlan {
    WeightedFreezePlan {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        budget: 0.0,
        weight: 0.0,
        phone: "".to_string(),
        status: WeightedFreezePlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_freeze_plan("code-77", "name-650");
        assert_eq!(item.status, WeightedFreezePlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_weighted_freeze_plan("code-77", "name-650");
        if item.move_to(WeightedFreezePlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFreezePlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_freeze_plan("code-77", "name-650");
        item.status = WeightedFreezePlanStatus::Failed;
        assert!(
            item.move_to(WeightedFreezePlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
