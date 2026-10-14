use super::*;

use acme_kit::text;

/// Where a adaptive rollback milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRollbackMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveRollbackMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveRollbackMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive rollback milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackMilestone {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub score: f64,
    pub approved: bool,
    pub retries: i64,
    pub rotation_days: i64,
    pub revision: i64,
    pub region: String,
    pub status: AdaptiveRollbackMilestoneStatus,
}

impl AdaptiveRollbackMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRollbackMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRollbackMilestoneStatus) -> bool {
        match self.status {
            AdaptiveRollbackMilestoneStatus::Created => {
                return next == AdaptiveRollbackMilestoneStatus::Validated;
            }
            AdaptiveRollbackMilestoneStatus::Validated => {
                return next == AdaptiveRollbackMilestoneStatus::Published
                    || next == AdaptiveRollbackMilestoneStatus::Retired;
            }
            AdaptiveRollbackMilestoneStatus::Published => {
                return next == AdaptiveRollbackMilestoneStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRollbackMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollback milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.revision as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRollbackMilestone {
        AdaptiveRollbackMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            score: self.score,
            approved: self.approved,
            retries: self.retries,
            rotation_days: self.rotation_days,
            revision: self.revision,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a adaptive rollback milestone in status created.
pub fn new_adaptive_rollback_milestone(code: &str, name: &str) -> AdaptiveRollbackMilestone {
    AdaptiveRollbackMilestone {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        score: 0.0,
        approved: false,
        retries: 0,
        rotation_days: 0,
        revision: 0,
        region: "".to_string(),
        status: AdaptiveRollbackMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_rollback_milestone("code-801", "name-295");
        assert_eq!(item.status, AdaptiveRollbackMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_adaptive_rollback_milestone("code-801", "name-295");
        if item.move_to(AdaptiveRollbackMilestoneStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRollbackMilestoneStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_rollback_milestone("code-801", "name-295");
        item.status = AdaptiveRollbackMilestoneStatus::Retired;
        assert!(
            item.move_to(AdaptiveRollbackMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
