use super::*;

use acme_kit::text;

/// Where a peering estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PeeringEstimateStatus {
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

impl std::fmt::Display for PeeringEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringEstimate {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub principal: String,
    pub cipher: String,
    pub rotation_days: i64,
    pub checksum: String,
    pub active: bool,
    pub duration_minutes: i64,
    pub segment: String,
    pub status: PeeringEstimateStatus,
}

impl PeeringEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringEstimateStatus) -> bool {
        match self.status {
            PeeringEstimateStatus::Idle => {
                match next {
                    PeeringEstimateStatus::Warming | PeeringEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringEstimateStatus::Warming => {
                match next {
                    PeeringEstimateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringEstimateStatus::Ready => {
                return next == PeeringEstimateStatus::Stopped;
            }
            PeeringEstimateStatus::Draining => {
                return next == PeeringEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rotation_days as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringEstimate {
        PeeringEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            cipher: self.cipher.clone(),
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            active: self.active,
            duration_minutes: self.duration_minutes,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a peering estimate in status idle.
pub fn new_peering_estimate(code: &str, title: &str) -> PeeringEstimate {
    PeeringEstimate {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        principal: "".to_string(),
        cipher: "".to_string(),
        rotation_days: 0,
        checksum: "".to_string(),
        active: false,
        duration_minutes: 0,
        segment: "".to_string(),
        status: PeeringEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_peering_estimate("code-972", "title-903");
        assert_eq!(item.status, PeeringEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_peering_estimate("code-972", "title-903");
        if item.move_to(PeeringEstimateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringEstimateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_peering_estimate("code-972", "title-903");
        item.status = PeeringEstimateStatus::Stopped;
        assert!(
            item.move_to(PeeringEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
