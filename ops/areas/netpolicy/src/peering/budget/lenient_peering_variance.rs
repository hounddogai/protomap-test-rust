use super::*;

use acme_kit::text;

/// Where a lenient peering variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPeeringVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientPeeringVarianceStatus {
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

impl std::fmt::Display for LenientPeeringVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient peering variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringVariance {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub amount: f64,
    pub discount: f64,
    pub status: LenientPeeringVarianceStatus,
}

impl LenientPeeringVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPeeringVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPeeringVarianceStatus) -> bool {
        match self.status {
            LenientPeeringVarianceStatus::Idle => {
                return next == LenientPeeringVarianceStatus::Ready;
            }
            LenientPeeringVarianceStatus::Warming => {
                match next {
                    LenientPeeringVarianceStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringVarianceStatus::Ready => {
                match next {
                    LenientPeeringVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringVarianceStatus::Draining => {
                match next {
                    LenientPeeringVarianceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientPeeringVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient peering variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient peering variance", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.discount * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPeeringVariance {
        LenientPeeringVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            amount: self.amount,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient peering variance in status idle.
pub fn new_lenient_peering_variance(code: &str, label: &str) -> LenientPeeringVariance {
    LenientPeeringVariance {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        amount: 0.0,
        discount: 0.0,
        status: LenientPeeringVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_peering_variance("code-636", "label-791");
        assert_eq!(item.status, LenientPeeringVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_peering_variance("code-636", "label-791");
        if item.move_to(LenientPeeringVarianceStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPeeringVarianceStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_peering_variance("code-636", "label-791");
        item.status = LenientPeeringVarianceStatus::Stopped;
        assert!(
            item.move_to(LenientPeeringVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
