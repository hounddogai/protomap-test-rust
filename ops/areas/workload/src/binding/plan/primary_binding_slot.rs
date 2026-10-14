use super::*;

use acme_kit::text;

/// Where a primary binding slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBindingSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryBindingSlotStatus {
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

impl std::fmt::Display for PrimaryBindingSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary binding slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlot {
    pub code: String,
    pub name: String,
    pub region: String,
    pub summary: String,
    pub window_days: i64,
    pub scope: String,
    pub status: PrimaryBindingSlotStatus,
}

impl PrimaryBindingSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBindingSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBindingSlotStatus) -> bool {
        match self.status {
            PrimaryBindingSlotStatus::Idle => {
                return next == PrimaryBindingSlotStatus::Warming;
            }
            PrimaryBindingSlotStatus::Warming => {
                match next {
                    PrimaryBindingSlotStatus::Draining | PrimaryBindingSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBindingSlotStatus::Ready => {
                return next == PrimaryBindingSlotStatus::Draining;
            }
            PrimaryBindingSlotStatus::Draining => {
                match next {
                    PrimaryBindingSlotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryBindingSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary binding slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> PrimaryBindingSlot {
        PrimaryBindingSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            summary: self.summary.clone(),
            window_days: self.window_days,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary binding slot in status idle.
pub fn new_primary_binding_slot(code: &str, name: &str) -> PrimaryBindingSlot {
    PrimaryBindingSlot {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        summary: "".to_string(),
        window_days: 0,
        scope: "".to_string(),
        status: PrimaryBindingSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_binding_slot("code-761", "name-474");
        assert_eq!(item.status, PrimaryBindingSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_primary_binding_slot("code-761", "name-474");
        if item.move_to(PrimaryBindingSlotStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryBindingSlotStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_binding_slot("code-761", "name-474");
        item.status = PrimaryBindingSlotStatus::Stopped;
        assert!(
            item.move_to(PrimaryBindingSlotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
