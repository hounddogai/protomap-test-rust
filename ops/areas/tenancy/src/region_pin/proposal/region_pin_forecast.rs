use super::*;

use acme_kit::text;

/// Where a region pin forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionPinForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionPinForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinForecast {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub ratio: f64,
    pub namespace: String,
    pub burst: i64,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub status: RegionPinForecastStatus,
}

impl RegionPinForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinForecastStatus) -> bool {
        match self.status {
            RegionPinForecastStatus::Draft => {
                return next == RegionPinForecastStatus::Active;
            }
            RegionPinForecastStatus::Pending => {
                return next == RegionPinForecastStatus::Active || next == RegionPinForecastStatus::Suspended;
            }
            RegionPinForecastStatus::Active => {
                match next {
                    RegionPinForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinForecastStatus::Suspended => {
                return next == RegionPinForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin forecast", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.ratio - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinForecast {
        RegionPinForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            ratio: self.ratio,
            namespace: self.namespace.clone(),
            burst: self.burst,
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a region pin forecast in status draft.
pub fn new_region_pin_forecast(code: &str, name: &str) -> RegionPinForecast {
    RegionPinForecast {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        ratio: 0.0,
        namespace: "".to_string(),
        burst: 0,
        grace_days: 0,
        tags: Vec::new(),
        status: RegionPinForecastStatus::Draft,
    }
}
