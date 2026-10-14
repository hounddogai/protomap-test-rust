use super::*;

use acme_kit::text;

/// Where a verifier allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl VerifierAllocationStatus {
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

impl std::fmt::Display for VerifierAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAllocation {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub tenant_id: String,
    pub quantity: i64,
    pub window_days: i64,
    pub tags: Vec<String>,
    pub status: VerifierAllocationStatus,
}

impl VerifierAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierAllocationStatus) -> bool {
        match self.status {
            VerifierAllocationStatus::Idle => {
                match next {
                    VerifierAllocationStatus::Draining | VerifierAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierAllocationStatus::Warming => {
                return next == VerifierAllocationStatus::Draining || next == VerifierAllocationStatus::Stopped;
            }
            VerifierAllocationStatus::Ready => {
                return next == VerifierAllocationStatus::Draining || next == VerifierAllocationStatus::Stopped;
            }
            VerifierAllocationStatus::Draining => {
                match next {
                    VerifierAllocationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: VerifierAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier allocation", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierAllocation {
        VerifierAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            window_days: self.window_days,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a verifier allocation in status idle.
pub fn new_verifier_allocation(code: &str, name: &str) -> VerifierAllocation {
    VerifierAllocation {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        tenant_id: "".to_string(),
        quantity: 0,
        window_days: 0,
        tags: Vec::new(),
        status: VerifierAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_verifier_allocation("code-351", "name-203");
        assert_eq!(item.status, VerifierAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_verifier_allocation("code-351", "name-203");
        if item.move_to(VerifierAllocationStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierAllocationStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_verifier_allocation("code-351", "name-203");
        item.status = VerifierAllocationStatus::Stopped;
        assert!(
            item.move_to(VerifierAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
