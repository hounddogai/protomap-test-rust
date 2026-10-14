use super::*;

use acme_kit::text;

/// Where a pending silence allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSilenceAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingSilenceAllocationStatus {
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

impl std::fmt::Display for PendingSilenceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending silence allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceAllocation {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub tags: Vec<String>,
    pub starts_at: i64,
    pub batch_size: i64,
    pub limit: i64,
    pub threshold: f64,
    pub tenant_id: String,
    pub approved: bool,
    pub ttl_seconds: i64,
    pub status: PendingSilenceAllocationStatus,
}

impl PendingSilenceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSilenceAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSilenceAllocationStatus) -> bool {
        match self.status {
            PendingSilenceAllocationStatus::Idle => {
                match next {
                    PendingSilenceAllocationStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSilenceAllocationStatus::Warming => {
                return next == PendingSilenceAllocationStatus::Ready
                    || next == PendingSilenceAllocationStatus::Draining;
            }
            PendingSilenceAllocationStatus::Ready => {
                return next == PendingSilenceAllocationStatus::Draining
                    || next == PendingSilenceAllocationStatus::Stopped;
            }
            PendingSilenceAllocationStatus::Draining => {
                match next {
                    PendingSilenceAllocationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingSilenceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending silence allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSilenceAllocation {
        PendingSilenceAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            tags: self.tags.clone(),
            starts_at: self.starts_at,
            batch_size: self.batch_size,
            limit: self.limit,
            threshold: self.threshold,
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending silence allocation in status idle.
pub fn new_pending_silence_allocation(code: &str, name: &str) -> PendingSilenceAllocation {
    PendingSilenceAllocation {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        tags: Vec::new(),
        starts_at: 0,
        batch_size: 0,
        limit: 0,
        threshold: 0.0,
        tenant_id: "".to_string(),
        approved: false,
        ttl_seconds: 0,
        status: PendingSilenceAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_silence_allocation("code-247", "name-417");
        assert_eq!(item.status, PendingSilenceAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_silence_allocation("code-247", "name-417");
        if item.move_to(PendingSilenceAllocationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSilenceAllocationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_silence_allocation("code-247", "name-417");
        item.status = PendingSilenceAllocationStatus::Stopped;
        assert!(
            item.move_to(PendingSilenceAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
