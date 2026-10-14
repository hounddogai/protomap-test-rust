use super::*;

use acme_kit::text;

/// Where a runbook tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookTierStatus {
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

impl std::fmt::Display for RunbookTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook tier as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTier {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub country: String,
    pub checksum: String,
    pub summary: String,
    pub score: f64,
    pub status: RunbookTierStatus,
}

impl RunbookTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookTierStatus) -> bool {
        match self.status {
            RunbookTierStatus::Planned => {
                return next == RunbookTierStatus::InProgress;
            }
            RunbookTierStatus::Scheduled => {
                return next == RunbookTierStatus::Completed;
            }
            RunbookTierStatus::InProgress => {
                match next {
                    RunbookTierStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookTierStatus::Completed => {
                match next {
                    RunbookTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RunbookTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RunbookTier {
        RunbookTier {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook tier in status planned.
pub fn new_runbook_tier(code: &str, label: &str) -> RunbookTier {
    RunbookTier {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        country: "".to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        score: 0.0,
        status: RunbookTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_runbook_tier("code-19", "label-331");
        assert_eq!(item.status, RunbookTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_runbook_tier("code-19", "label-331");
        if item.move_to(RunbookTierStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookTierStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_runbook_tier("code-19", "label-331");
        item.status = RunbookTierStatus::Cancelled;
        assert!(
            item.move_to(RunbookTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
