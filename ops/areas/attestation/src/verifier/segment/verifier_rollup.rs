use super::*;

use acme_kit::text;

/// Where a verifier rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierRollupStatus {
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

impl std::fmt::Display for VerifierRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRollup {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub approved: bool,
    pub checksum: String,
    pub status: VerifierRollupStatus,
}

impl VerifierRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierRollupStatus) -> bool {
        match self.status {
            VerifierRollupStatus::Planned => {
                match next {
                    VerifierRollupStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierRollupStatus::Scheduled => {
                return next == VerifierRollupStatus::Cancelled;
            }
            VerifierRollupStatus::InProgress => {
                return next == VerifierRollupStatus::Completed || next == VerifierRollupStatus::Cancelled;
            }
            VerifierRollupStatus::Completed => {
                return next == VerifierRollupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> VerifierRollup {
        VerifierRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            approved: self.approved,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a verifier rollup in status planned.
pub fn new_verifier_rollup(code: &str, name: &str) -> VerifierRollup {
    VerifierRollup {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        approved: false,
        checksum: "".to_string(),
        status: VerifierRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_verifier_rollup("code-694", "name-353");
        assert_eq!(item.status, VerifierRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_verifier_rollup("code-694", "name-353");
        if item.move_to(VerifierRollupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierRollupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_verifier_rollup("code-694", "name-353");
        item.status = VerifierRollupStatus::Cancelled;
        assert!(
            item.move_to(VerifierRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
