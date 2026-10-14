use super::*;

use acme_kit::text;

/// Where a seasonal health check baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalHealthCheckBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalHealthCheckBaselineStatus {
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

impl std::fmt::Display for SeasonalHealthCheckBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal health check baseline as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaseline {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub sequence: i64,
    pub retries: i64,
    pub locale: String,
    pub cost: f64,
    pub burst: i64,
    pub checksum: String,
    pub label: String,
    pub owner_id: String,
    pub status: SeasonalHealthCheckBaselineStatus,
}

impl SeasonalHealthCheckBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalHealthCheckBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalHealthCheckBaselineStatus) -> bool {
        match self.status {
            SeasonalHealthCheckBaselineStatus::Idle => {
                match next {
                    SeasonalHealthCheckBaselineStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalHealthCheckBaselineStatus::Warming => {
                match next {
                    SeasonalHealthCheckBaselineStatus::Ready | SeasonalHealthCheckBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalHealthCheckBaselineStatus::Ready => {
                return next == SeasonalHealthCheckBaselineStatus::Draining
                    || next == SeasonalHealthCheckBaselineStatus::Stopped;
            }
            SeasonalHealthCheckBaselineStatus::Draining => {
                return next == SeasonalHealthCheckBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalHealthCheckBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal health check baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal health check baseline", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.burst as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalHealthCheckBaseline {
        SeasonalHealthCheckBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            sequence: self.sequence,
            retries: self.retries,
            locale: self.locale.clone(),
            cost: self.cost,
            burst: self.burst,
            checksum: self.checksum.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a seasonal health check baseline in status idle.
pub fn new_seasonal_health_check_baseline(code: &str, title: &str) -> SeasonalHealthCheckBaseline {
    SeasonalHealthCheckBaseline {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        sequence: 0,
        retries: 0,
        locale: "".to_string(),
        cost: 0.0,
        burst: 0,
        checksum: "".to_string(),
        label: "".to_string(),
        owner_id: "".to_string(),
        status: SeasonalHealthCheckBaselineStatus::Idle,
    }
}
