use super::*;

use acme_kit::text;

/// Where a peering adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PeeringAdjustmentStatus {
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

impl std::fmt::Display for PeeringAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering adjustment as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAdjustment {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub cipher: String,
    pub owner_id: String,
    pub issuer: String,
    pub amount: f64,
    pub status: PeeringAdjustmentStatus,
}

impl PeeringAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringAdjustmentStatus) -> bool {
        match self.status {
            PeeringAdjustmentStatus::Idle => {
                return next == PeeringAdjustmentStatus::Warming || next == PeeringAdjustmentStatus::Stopped;
            }
            PeeringAdjustmentStatus::Warming => {
                return next == PeeringAdjustmentStatus::Ready || next == PeeringAdjustmentStatus::Draining;
            }
            PeeringAdjustmentStatus::Ready => {
                match next {
                    PeeringAdjustmentStatus::Draining | PeeringAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringAdjustmentStatus::Draining => {
                match next {
                    PeeringAdjustmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PeeringAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> PeeringAdjustment {
        PeeringAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            cipher: self.cipher.clone(),
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a peering adjustment in status idle.
pub fn new_peering_adjustment(code: &str, name: &str) -> PeeringAdjustment {
    PeeringAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        cipher: "".to_string(),
        owner_id: "".to_string(),
        issuer: "".to_string(),
        amount: 0.0,
        status: PeeringAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_peering_adjustment("code-295", "name-68");
        assert_eq!(item.status, PeeringAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_peering_adjustment("code-295", "name-68");
        if item.move_to(PeeringAdjustmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringAdjustmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_peering_adjustment("code-295", "name-68");
        item.status = PeeringAdjustmentStatus::Stopped;
        assert!(
            item.move_to(PeeringAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
