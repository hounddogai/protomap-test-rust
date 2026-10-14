use super::*;

use acme_kit::text;

/// Where a region pin estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionPinEstimateStatus {
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

impl std::fmt::Display for RegionPinEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin estimate as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimate {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub created_at: i64,
    pub issuer: String,
    pub title: String,
    pub score: f64,
    pub phone: String,
    pub status: RegionPinEstimateStatus,
}

impl RegionPinEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinEstimateStatus) -> bool {
        match self.status {
            RegionPinEstimateStatus::Idle => {
                return next == RegionPinEstimateStatus::Ready || next == RegionPinEstimateStatus::Draining;
            }
            RegionPinEstimateStatus::Warming => {
                return next == RegionPinEstimateStatus::Ready || next == RegionPinEstimateStatus::Draining;
            }
            RegionPinEstimateStatus::Ready => {
                return next == RegionPinEstimateStatus::Draining || next == RegionPinEstimateStatus::Stopped;
            }
            RegionPinEstimateStatus::Draining => {
                return next == RegionPinEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "region pin estimate", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinEstimate {
        RegionPinEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            score: self.score,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin estimate in status idle.
pub fn new_region_pin_estimate(code: &str, label: &str) -> RegionPinEstimate {
    RegionPinEstimate {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        created_at: 0,
        issuer: "".to_string(),
        title: "".to_string(),
        score: 0.0,
        phone: "".to_string(),
        status: RegionPinEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_region_pin_estimate("code-809", "label-492");
        assert_eq!(item.status, RegionPinEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_region_pin_estimate("code-809", "label-492");
        if item.move_to(RegionPinEstimateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinEstimateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_region_pin_estimate("code-809", "label-492");
        item.status = RegionPinEstimateStatus::Stopped;
        assert!(
            item.move_to(RegionPinEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
