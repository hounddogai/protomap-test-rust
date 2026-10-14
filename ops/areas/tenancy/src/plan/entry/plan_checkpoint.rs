use super::*;

use acme_kit::text;

/// Where a plan checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PlanCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PlanCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan checkpoint as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanCheckpoint {
    pub code: String,
    pub label: String,
    pub category: String,
    pub cost: f64,
    pub rotation_days: i64,
    pub ratio: f64,
    pub status: PlanCheckpointStatus,
}

impl PlanCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanCheckpointStatus) -> bool {
        match self.status {
            PlanCheckpointStatus::Draft => {
                return next == PlanCheckpointStatus::Pending || next == PlanCheckpointStatus::Active;
            }
            PlanCheckpointStatus::Pending => {
                match next {
                    PlanCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanCheckpointStatus::Active => {
                match next {
                    PlanCheckpointStatus::Suspended | PlanCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanCheckpointStatus::Suspended => {
                return next == PlanCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "plan checkpoint", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.cost * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanCheckpoint {
        PlanCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            cost: self.cost,
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a plan checkpoint in status draft.
pub fn new_plan_checkpoint(code: &str, label: &str) -> PlanCheckpoint {
    PlanCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        cost: 0.0,
        rotation_days: 0,
        ratio: 0.0,
        status: PlanCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_plan_checkpoint("code-742", "label-9");
        assert_eq!(item.status, PlanCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_plan_checkpoint("code-742", "label-9");
        if item.move_to(PlanCheckpointStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanCheckpointStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_plan_checkpoint("code-742", "label-9");
        item.status = PlanCheckpointStatus::Closed;
        assert!(
            item.move_to(PlanCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
