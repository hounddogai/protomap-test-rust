use super::*;

use acme_kit::text;

/// Where a overage rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl OverageRollupStatus {
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

impl std::fmt::Display for OverageRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageRollup {
    pub code: String,
    pub name: String,
    pub category: String,
    pub audience: String,
    pub revision: i64,
    pub status: OverageRollupStatus,
}

impl OverageRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageRollupStatus) -> bool {
        match self.status {
            OverageRollupStatus::Planned => {
                return next == OverageRollupStatus::InProgress;
            }
            OverageRollupStatus::Scheduled => {
                match next {
                    OverageRollupStatus::InProgress | OverageRollupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageRollupStatus::InProgress => {
                match next {
                    OverageRollupStatus::Completed | OverageRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageRollupStatus::Completed => {
                match next {
                    OverageRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: OverageRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "overage rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> OverageRollup {
        OverageRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            audience: self.audience.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a overage rollup in status planned.
pub fn new_overage_rollup(code: &str, name: &str) -> OverageRollup {
    OverageRollup {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        audience: "".to_string(),
        revision: 0,
        status: OverageRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_overage_rollup("code-45", "name-683");
        assert_eq!(item.status, OverageRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_overage_rollup("code-45", "name-683");
        if item.move_to(OverageRollupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageRollupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_overage_rollup("code-45", "name-683");
        item.status = OverageRollupStatus::Cancelled;
        assert!(
            item.move_to(OverageRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
