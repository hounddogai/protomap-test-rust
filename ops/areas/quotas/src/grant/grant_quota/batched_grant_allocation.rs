use super::*;

use acme_kit::text;

/// Where a batched grant allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedGrantAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedGrantAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedGrantAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched grant allocation as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantAllocation {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub principal: String,
    pub reviewer: String,
    pub weight: f64,
    pub status: BatchedGrantAllocationStatus,
}

impl BatchedGrantAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedGrantAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedGrantAllocationStatus) -> bool {
        match self.status {
            BatchedGrantAllocationStatus::Idle => {
                return next == BatchedGrantAllocationStatus::Ready;
            }
            BatchedGrantAllocationStatus::Warming => {
                match next {
                    BatchedGrantAllocationStatus::Ready | BatchedGrantAllocationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedGrantAllocationStatus::Ready => {
                return next == BatchedGrantAllocationStatus::Draining;
            }
            BatchedGrantAllocationStatus::Draining => {
                match next {
                    BatchedGrantAllocationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedGrantAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched grant allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched grant allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> BatchedGrantAllocation {
        BatchedGrantAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            reviewer: self.reviewer.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched grant allocation in status idle.
pub fn new_batched_grant_allocation(code: &str, title: &str) -> BatchedGrantAllocation {
    BatchedGrantAllocation {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        principal: "".to_string(),
        reviewer: "".to_string(),
        weight: 0.0,
        status: BatchedGrantAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_grant_allocation("code-530", "title-510");
        assert_eq!(item.status, BatchedGrantAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_batched_grant_allocation("code-530", "title-510");
        if item.move_to(BatchedGrantAllocationStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedGrantAllocationStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_grant_allocation("code-530", "title-510");
        item.status = BatchedGrantAllocationStatus::Stopped;
        assert!(
            item.move_to(BatchedGrantAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
