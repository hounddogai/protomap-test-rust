use super::*;

use acme_kit::text;

/// Where a tiered postmortem override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPostmortemOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredPostmortemOverrideStatus {
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

impl std::fmt::Display for TieredPostmortemOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered postmortem override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemOverride {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub batch_size: i64,
    pub unit: String,
    pub summary: String,
    pub status: TieredPostmortemOverrideStatus,
}

impl TieredPostmortemOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPostmortemOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPostmortemOverrideStatus) -> bool {
        match self.status {
            TieredPostmortemOverrideStatus::Planned => {
                return next == TieredPostmortemOverrideStatus::InProgress
                    || next == TieredPostmortemOverrideStatus::Cancelled;
            }
            TieredPostmortemOverrideStatus::Scheduled => {
                return next == TieredPostmortemOverrideStatus::Completed
                    || next == TieredPostmortemOverrideStatus::Cancelled;
            }
            TieredPostmortemOverrideStatus::InProgress => {
                return next == TieredPostmortemOverrideStatus::Completed
                    || next == TieredPostmortemOverrideStatus::Cancelled;
            }
            TieredPostmortemOverrideStatus::Completed => {
                match next {
                    TieredPostmortemOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredPostmortemOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered postmortem override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPostmortemOverride {
        TieredPostmortemOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            batch_size: self.batch_size,
            unit: self.unit.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered postmortem override in status planned.
pub fn new_tiered_postmortem_override(code: &str, name: &str) -> TieredPostmortemOverride {
    TieredPostmortemOverride {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        batch_size: 0,
        unit: "".to_string(),
        summary: "".to_string(),
        status: TieredPostmortemOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_postmortem_override("code-936", "name-473");
        assert_eq!(item.status, TieredPostmortemOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_postmortem_override("code-936", "name-473");
        if item.move_to(TieredPostmortemOverrideStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPostmortemOverrideStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_postmortem_override("code-936", "name-473");
        item.status = TieredPostmortemOverrideStatus::Cancelled;
        assert!(
            item.move_to(TieredPostmortemOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
