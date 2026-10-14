use super::*;

use acme_kit::text;

/// Where a fallback data export estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDataExportEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackDataExportEstimateStatus {
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

impl std::fmt::Display for FallbackDataExportEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback data export estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportEstimate {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub phone: String,
    pub margin: f64,
    pub principal: String,
    pub threshold: f64,
    pub rotation_days: i64,
    pub status: FallbackDataExportEstimateStatus,
}

impl FallbackDataExportEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDataExportEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDataExportEstimateStatus) -> bool {
        match self.status {
            FallbackDataExportEstimateStatus::Idle => {
                match next {
                    FallbackDataExportEstimateStatus::Ready | FallbackDataExportEstimateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportEstimateStatus::Warming => {
                match next {
                    FallbackDataExportEstimateStatus::Ready | FallbackDataExportEstimateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportEstimateStatus::Ready => {
                return next == FallbackDataExportEstimateStatus::Draining;
            }
            FallbackDataExportEstimateStatus::Draining => {
                match next {
                    FallbackDataExportEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackDataExportEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback data export estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDataExportEstimate {
        FallbackDataExportEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            phone: self.phone.clone(),
            margin: self.margin,
            principal: self.principal.clone(),
            threshold: self.threshold,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback data export estimate in status idle.
pub fn new_fallback_data_export_estimate(code: &str, label: &str) -> FallbackDataExportEstimate {
    FallbackDataExportEstimate {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        phone: "".to_string(),
        margin: 0.0,
        principal: "".to_string(),
        threshold: 0.0,
        rotation_days: 0,
        status: FallbackDataExportEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_data_export_estimate("code-668", "label-282");
        assert_eq!(item.status, FallbackDataExportEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_fallback_data_export_estimate("code-668", "label-282");
        if item.move_to(FallbackDataExportEstimateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDataExportEstimateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_data_export_estimate("code-668", "label-282");
        item.status = FallbackDataExportEstimateStatus::Stopped;
        assert!(
            item.move_to(FallbackDataExportEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
