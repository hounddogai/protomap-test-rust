use super::*;

use acme_kit::text;

/// Where a primary postmortem target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPostmortemTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryPostmortemTargetStatus {
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

impl std::fmt::Display for PrimaryPostmortemTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary postmortem target as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemTarget {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub summary: String,
    pub segment: String,
    pub threshold: f64,
    pub status: PrimaryPostmortemTargetStatus,
}

impl PrimaryPostmortemTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPostmortemTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPostmortemTargetStatus) -> bool {
        match self.status {
            PrimaryPostmortemTargetStatus::Planned => {
                match next {
                    PrimaryPostmortemTargetStatus::Scheduled | PrimaryPostmortemTargetStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPostmortemTargetStatus::Scheduled => {
                match next {
                    PrimaryPostmortemTargetStatus::Completed | PrimaryPostmortemTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPostmortemTargetStatus::InProgress => {
                return next == PrimaryPostmortemTargetStatus::Completed
                    || next == PrimaryPostmortemTargetStatus::Cancelled;
            }
            PrimaryPostmortemTargetStatus::Completed => {
                return next == PrimaryPostmortemTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPostmortemTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary postmortem target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.threshold * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPostmortemTarget {
        PrimaryPostmortemTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            summary: self.summary.clone(),
            segment: self.segment.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary postmortem target in status planned.
pub fn new_primary_postmortem_target(code: &str, name: &str) -> PrimaryPostmortemTarget {
    PrimaryPostmortemTarget {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        summary: "".to_string(),
        segment: "".to_string(),
        threshold: 0.0,
        status: PrimaryPostmortemTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_postmortem_target("code-743", "name-808");
        assert_eq!(item.status, PrimaryPostmortemTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_postmortem_target("code-743", "name-808");
        if item.move_to(PrimaryPostmortemTargetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPostmortemTargetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_postmortem_target("code-743", "name-808");
        item.status = PrimaryPostmortemTargetStatus::Cancelled;
        assert!(
            item.move_to(PrimaryPostmortemTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
