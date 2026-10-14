use super::*;

use acme_kit::text;

/// Where a peering target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PeeringTargetStatus {
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

impl std::fmt::Display for PeeringTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTarget {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub locale: String,
    pub amount: f64,
    pub batch_size: i64,
    pub host: String,
    pub reviewer: String,
    pub discount: f64,
    pub status: PeeringTargetStatus,
}

impl PeeringTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringTargetStatus) -> bool {
        match self.status {
            PeeringTargetStatus::Idle => {
                return next == PeeringTargetStatus::Ready || next == PeeringTargetStatus::Stopped;
            }
            PeeringTargetStatus::Warming => {
                return next == PeeringTargetStatus::Ready;
            }
            PeeringTargetStatus::Ready => {
                return next == PeeringTargetStatus::Stopped;
            }
            PeeringTargetStatus::Draining => {
                return next == PeeringTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.amount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringTarget {
        PeeringTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            amount: self.amount,
            batch_size: self.batch_size,
            host: self.host.clone(),
            reviewer: self.reviewer.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a peering target in status idle.
pub fn new_peering_target(code: &str, label: &str) -> PeeringTarget {
    PeeringTarget {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        locale: "".to_string(),
        amount: 0.0,
        batch_size: 0,
        host: "".to_string(),
        reviewer: "".to_string(),
        discount: 0.0,
        status: PeeringTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_peering_target("code-548", "label-941");
        assert_eq!(item.status, PeeringTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_peering_target("code-548", "label-941");
        if item.move_to(PeeringTargetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringTargetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_peering_target("code-548", "label-941");
        item.status = PeeringTargetStatus::Stopped;
        assert!(
            item.move_to(PeeringTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
