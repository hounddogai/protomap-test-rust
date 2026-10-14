use super::*;

use acme_kit::text;

/// Where a tiered plan profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPlanProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredPlanProfileStatus {
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

impl std::fmt::Display for TieredPlanProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered plan profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanProfile {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub weight: f64,
    pub fingerprint: String,
    pub phone: String,
    pub owner_id: String,
    pub category: String,
    pub status: TieredPlanProfileStatus,
}

impl TieredPlanProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPlanProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPlanProfileStatus) -> bool {
        match self.status {
            TieredPlanProfileStatus::New => {
                return next == TieredPlanProfileStatus::Queued || next == TieredPlanProfileStatus::Failed;
            }
            TieredPlanProfileStatus::Queued => {
                return next == TieredPlanProfileStatus::Failed;
            }
            TieredPlanProfileStatus::Running => {
                return next == TieredPlanProfileStatus::Failed;
            }
            TieredPlanProfileStatus::Done => {
                match next {
                    TieredPlanProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredPlanProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered plan profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered plan profile", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.weight + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPlanProfile {
        TieredPlanProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            weight: self.weight,
            fingerprint: self.fingerprint.clone(),
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tiered plan profile in status new.
pub fn new_tiered_plan_profile(code: &str, label: &str) -> TieredPlanProfile {
    TieredPlanProfile {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        weight: 0.0,
        fingerprint: "".to_string(),
        phone: "".to_string(),
        owner_id: "".to_string(),
        category: "".to_string(),
        status: TieredPlanProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_plan_profile("code-272", "label-602");
        assert_eq!(item.status, TieredPlanProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_plan_profile("code-272", "label-602");
        if item.move_to(TieredPlanProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPlanProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_plan_profile("code-272", "label-602");
        item.status = TieredPlanProfileStatus::Failed;
        assert!(
            item.move_to(TieredPlanProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
