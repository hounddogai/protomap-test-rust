use super::*;

use acme_kit::text;

/// Where a weighted rollback bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRollbackBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedRollbackBundleStatus {
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

impl std::fmt::Display for WeightedRollbackBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted rollback bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundle {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub key_id: String,
    pub summary: String,
    pub issuer: String,
    pub segment: String,
    pub source: String,
    pub score: f64,
    pub status: WeightedRollbackBundleStatus,
}

impl WeightedRollbackBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRollbackBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRollbackBundleStatus) -> bool {
        match self.status {
            WeightedRollbackBundleStatus::Planned => {
                return next == WeightedRollbackBundleStatus::Completed;
            }
            WeightedRollbackBundleStatus::Scheduled => {
                return next == WeightedRollbackBundleStatus::Completed
                    || next == WeightedRollbackBundleStatus::Cancelled;
            }
            WeightedRollbackBundleStatus::InProgress => {
                match next {
                    WeightedRollbackBundleStatus::Completed | WeightedRollbackBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRollbackBundleStatus::Completed => {
                return next == WeightedRollbackBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRollbackBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rollback bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> WeightedRollbackBundle {
        WeightedRollbackBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            key_id: self.key_id.clone(),
            summary: self.summary.clone(),
            issuer: self.issuer.clone(),
            segment: self.segment.clone(),
            source: self.source.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted rollback bundle in status planned.
pub fn new_weighted_rollback_bundle(code: &str, name: &str) -> WeightedRollbackBundle {
    WeightedRollbackBundle {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        key_id: "".to_string(),
        summary: "".to_string(),
        issuer: "".to_string(),
        segment: "".to_string(),
        source: "".to_string(),
        score: 0.0,
        status: WeightedRollbackBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_rollback_bundle("code-545", "name-9");
        assert_eq!(item.status, WeightedRollbackBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_rollback_bundle("code-545", "name-9");
        if item.move_to(WeightedRollbackBundleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRollbackBundleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_rollback_bundle("code-545", "name-9");
        item.status = WeightedRollbackBundleStatus::Cancelled;
        assert!(
            item.move_to(WeightedRollbackBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
