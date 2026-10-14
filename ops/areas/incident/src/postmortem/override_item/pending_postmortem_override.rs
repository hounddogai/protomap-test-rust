use super::*;

use acme_kit::text;

/// Where a pending postmortem override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPostmortemOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingPostmortemOverrideStatus {
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

impl std::fmt::Display for PendingPostmortemOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending postmortem override as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemOverride {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub channel: String,
    pub audience: String,
    pub quantity: i64,
    pub status: PendingPostmortemOverrideStatus,
}

impl PendingPostmortemOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPostmortemOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPostmortemOverrideStatus) -> bool {
        match self.status {
            PendingPostmortemOverrideStatus::Idle => {
                match next {
                    PendingPostmortemOverrideStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPostmortemOverrideStatus::Warming => {
                match next {
                    PendingPostmortemOverrideStatus::Ready | PendingPostmortemOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPostmortemOverrideStatus::Ready => {
                return next == PendingPostmortemOverrideStatus::Draining
                    || next == PendingPostmortemOverrideStatus::Stopped;
            }
            PendingPostmortemOverrideStatus::Draining => {
                return next == PendingPostmortemOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPostmortemOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending postmortem override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PendingPostmortemOverride {
        PendingPostmortemOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending postmortem override in status idle.
pub fn new_pending_postmortem_override(code: &str, title: &str) -> PendingPostmortemOverride {
    PendingPostmortemOverride {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        channel: "".to_string(),
        audience: "".to_string(),
        quantity: 0,
        status: PendingPostmortemOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_postmortem_override("code-679", "title-667");
        assert_eq!(item.status, PendingPostmortemOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_postmortem_override("code-679", "title-667");
        if item.move_to(PendingPostmortemOverrideStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPostmortemOverrideStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_postmortem_override("code-679", "title-667");
        item.status = PendingPostmortemOverrideStatus::Stopped;
        assert!(
            item.move_to(PendingPostmortemOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
