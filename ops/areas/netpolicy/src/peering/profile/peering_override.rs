use super::*;

use acme_kit::text;

/// Where a peering override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PeeringOverrideStatus {
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

impl std::fmt::Display for PeeringOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering override as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringOverride {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub rate: f64,
    pub starts_at: i64,
    pub audience: String,
    pub status: PeeringOverrideStatus,
}

impl PeeringOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringOverrideStatus) -> bool {
        match self.status {
            PeeringOverrideStatus::Idle => {
                match next {
                    PeeringOverrideStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringOverrideStatus::Warming => {
                match next {
                    PeeringOverrideStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringOverrideStatus::Ready => {
                return next == PeeringOverrideStatus::Draining || next == PeeringOverrideStatus::Stopped;
            }
            PeeringOverrideStatus::Draining => {
                match next {
                    PeeringOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PeeringOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering override {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.starts_at as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringOverride {
        PeeringOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            rate: self.rate,
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a peering override in status idle.
pub fn new_peering_override(code: &str, label: &str) -> PeeringOverride {
    PeeringOverride {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        rate: 0.0,
        starts_at: 0,
        audience: "".to_string(),
        status: PeeringOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_peering_override("code-527", "label-636");
        assert_eq!(item.status, PeeringOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_peering_override("code-527", "label-636");
        if item.move_to(PeeringOverrideStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringOverrideStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_peering_override("code-527", "label-636");
        item.status = PeeringOverrideStatus::Stopped;
        assert!(
            item.move_to(PeeringOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
