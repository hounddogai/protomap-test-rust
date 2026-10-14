use super::*;

use acme_kit::text;

/// Where a silence allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SilenceAllocationStatus {
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

impl std::fmt::Display for SilenceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocation {
    pub code: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub amount: f64,
    pub status: SilenceAllocationStatus,
}

impl SilenceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceAllocationStatus) -> bool {
        match self.status {
            SilenceAllocationStatus::New => {
                match next {
                    SilenceAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceAllocationStatus::Queued => {
                return next == SilenceAllocationStatus::Done;
            }
            SilenceAllocationStatus::Running => {
                return next == SilenceAllocationStatus::Done;
            }
            SilenceAllocationStatus::Done => {
                match next {
                    SilenceAllocationStatus::Failed => {
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

    pub fn move_to(&mut self, next: SilenceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> SilenceAllocation {
        SilenceAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a silence allocation in status new.
pub fn new_silence_allocation(code: &str, name: &str) -> SilenceAllocation {
    SilenceAllocation {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        owner_id: "".to_string(),
        amount: 0.0,
        status: SilenceAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_silence_allocation("code-608", "name-810");
        assert_eq!(item.status, SilenceAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_silence_allocation("code-608", "name-810");
        if item.move_to(SilenceAllocationStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceAllocationStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_silence_allocation("code-608", "name-810");
        item.status = SilenceAllocationStatus::Failed;
        assert!(
            item.move_to(SilenceAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
