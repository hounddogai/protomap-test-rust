use super::*;

use acme_kit::text;

/// Where a limit allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitAllocationStatus {
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

impl std::fmt::Display for LimitAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAllocation {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub issuer: String,
    pub ceiling: f64,
    pub namespace: String,
    pub status: LimitAllocationStatus,
}

impl LimitAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitAllocationStatus) -> bool {
        match self.status {
            LimitAllocationStatus::Idle => {
                match next {
                    LimitAllocationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitAllocationStatus::Warming => {
                return next == LimitAllocationStatus::Draining || next == LimitAllocationStatus::Stopped;
            }
            LimitAllocationStatus::Ready => {
                return next == LimitAllocationStatus::Draining;
            }
            LimitAllocationStatus::Draining => {
                match next {
                    LimitAllocationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LimitAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitAllocation {
        LimitAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a limit allocation in status idle.
pub fn new_limit_allocation(code: &str, name: &str) -> LimitAllocation {
    LimitAllocation {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        issuer: "".to_string(),
        ceiling: 0.0,
        namespace: "".to_string(),
        status: LimitAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_limit_allocation("code-879", "name-395");
        assert_eq!(item.status, LimitAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_limit_allocation("code-879", "name-395");
        if item.move_to(LimitAllocationStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitAllocationStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_limit_allocation("code-879", "name-395");
        item.status = LimitAllocationStatus::Stopped;
        assert!(
            item.move_to(LimitAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
