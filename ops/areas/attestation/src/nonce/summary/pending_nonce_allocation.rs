use super::*;

use acme_kit::text;

/// Where a pending nonce allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingNonceAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingNonceAllocationStatus {
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

impl std::fmt::Display for PendingNonceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending nonce allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceAllocation {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub sequence: i64,
    pub phone: String,
    pub source: String,
    pub namespace: String,
    pub capacity: i64,
    pub amount: f64,
    pub port: i64,
    pub approved: bool,
    pub status: PendingNonceAllocationStatus,
}

impl PendingNonceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingNonceAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingNonceAllocationStatus) -> bool {
        match self.status {
            PendingNonceAllocationStatus::Idle => {
                return next == PendingNonceAllocationStatus::Ready;
            }
            PendingNonceAllocationStatus::Warming => {
                match next {
                    PendingNonceAllocationStatus::Draining | PendingNonceAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceAllocationStatus::Ready => {
                match next {
                    PendingNonceAllocationStatus::Draining | PendingNonceAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceAllocationStatus::Draining => {
                match next {
                    PendingNonceAllocationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingNonceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending nonce allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.amount - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingNonceAllocation {
        PendingNonceAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            sequence: self.sequence,
            phone: self.phone.clone(),
            source: self.source.clone(),
            namespace: self.namespace.clone(),
            capacity: self.capacity,
            amount: self.amount,
            port: self.port,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending nonce allocation in status idle.
pub fn new_pending_nonce_allocation(code: &str, title: &str) -> PendingNonceAllocation {
    PendingNonceAllocation {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        sequence: 0,
        phone: "".to_string(),
        source: "".to_string(),
        namespace: "".to_string(),
        capacity: 0,
        amount: 0.0,
        port: 0,
        approved: false,
        status: PendingNonceAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_nonce_allocation("code-678", "title-345");
        assert_eq!(item.status, PendingNonceAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_pending_nonce_allocation("code-678", "title-345");
        if item.move_to(PendingNonceAllocationStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingNonceAllocationStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_nonce_allocation("code-678", "title-345");
        item.status = PendingNonceAllocationStatus::Stopped;
        assert!(
            item.move_to(PendingNonceAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
