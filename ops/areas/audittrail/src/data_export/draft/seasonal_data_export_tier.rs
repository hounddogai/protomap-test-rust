use super::*;

use acme_kit::text;

/// Where a seasonal data export tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDataExportTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalDataExportTierStatus {
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

impl std::fmt::Display for SeasonalDataExportTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal data export tier as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportTier {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub grace_days: i64,
    pub ttl_seconds: i64,
    pub status: SeasonalDataExportTierStatus,
}

impl SeasonalDataExportTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDataExportTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDataExportTierStatus) -> bool {
        match self.status {
            SeasonalDataExportTierStatus::Idle => {
                return next == SeasonalDataExportTierStatus::Draining || next == SeasonalDataExportTierStatus::Stopped;
            }
            SeasonalDataExportTierStatus::Warming => {
                return next == SeasonalDataExportTierStatus::Stopped;
            }
            SeasonalDataExportTierStatus::Ready => {
                return next == SeasonalDataExportTierStatus::Draining;
            }
            SeasonalDataExportTierStatus::Draining => {
                return next == SeasonalDataExportTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDataExportTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal data export tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal data export tier", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.grace_days as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDataExportTier {
        SeasonalDataExportTier {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            grace_days: self.grace_days,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal data export tier in status idle.
pub fn new_seasonal_data_export_tier(code: &str, title: &str) -> SeasonalDataExportTier {
    SeasonalDataExportTier {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        grace_days: 0,
        ttl_seconds: 0,
        status: SeasonalDataExportTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_data_export_tier("code-615", "title-343");
        assert_eq!(item.status, SeasonalDataExportTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_data_export_tier("code-615", "title-343");
        if item.move_to(SeasonalDataExportTierStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDataExportTierStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_data_export_tier("code-615", "title-343");
        item.status = SeasonalDataExportTierStatus::Stopped;
        assert!(
            item.move_to(SeasonalDataExportTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
