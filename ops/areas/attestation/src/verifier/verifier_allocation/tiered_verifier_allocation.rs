use super::*;

use acme_kit::text;

/// Where a tiered verifier allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredVerifierAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredVerifierAllocationStatus {
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

impl std::fmt::Display for TieredVerifierAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered verifier allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierAllocation {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub sequence: i64,
    pub created_at: i64,
    pub audience: String,
    pub batch_size: i64,
    pub status: TieredVerifierAllocationStatus,
}

impl TieredVerifierAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredVerifierAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredVerifierAllocationStatus) -> bool {
        match self.status {
            TieredVerifierAllocationStatus::New => {
                match next {
                    TieredVerifierAllocationStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredVerifierAllocationStatus::Queued => {
                match next {
                    TieredVerifierAllocationStatus::Running | TieredVerifierAllocationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredVerifierAllocationStatus::Running => {
                return next == TieredVerifierAllocationStatus::Done || next == TieredVerifierAllocationStatus::Failed;
            }
            TieredVerifierAllocationStatus::Done => {
                return next == TieredVerifierAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredVerifierAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered verifier allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered verifier allocation", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.batch_size as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredVerifierAllocation {
        TieredVerifierAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            sequence: self.sequence,
            created_at: self.created_at,
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered verifier allocation in status new.
pub fn new_tiered_verifier_allocation(code: &str, name: &str) -> TieredVerifierAllocation {
    TieredVerifierAllocation {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        sequence: 0,
        created_at: 0,
        audience: "".to_string(),
        batch_size: 0,
        status: TieredVerifierAllocationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_verifier_allocation("code-85", "name-518");
        assert_eq!(item.status, TieredVerifierAllocationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_verifier_allocation("code-85", "name-518");
        if item.move_to(TieredVerifierAllocationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredVerifierAllocationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_verifier_allocation("code-85", "name-518");
        item.status = TieredVerifierAllocationStatus::Failed;
        assert!(
            item.move_to(TieredVerifierAllocationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
