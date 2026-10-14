use super::*;

use acme_kit::text;

/// Where a adaptive forecast segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveForecastSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveForecastSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveForecastSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive forecast segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastSegment {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub host: String,
    pub window_days: i64,
    pub active: bool,
    pub archived: bool,
    pub tags: Vec<String>,
    pub status: AdaptiveForecastSegmentStatus,
}

impl AdaptiveForecastSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveForecastSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveForecastSegmentStatus) -> bool {
        match self.status {
            AdaptiveForecastSegmentStatus::Requested => {
                return next == AdaptiveForecastSegmentStatus::Expired;
            }
            AdaptiveForecastSegmentStatus::Granted => {
                return next == AdaptiveForecastSegmentStatus::Revoked
                    || next == AdaptiveForecastSegmentStatus::Expired;
            }
            AdaptiveForecastSegmentStatus::Revoked => {
                match next {
                    AdaptiveForecastSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptiveForecastSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive forecast segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive forecast segment", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> AdaptiveForecastSegment {
        AdaptiveForecastSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            host: self.host.clone(),
            window_days: self.window_days,
            active: self.active,
            archived: self.archived,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive forecast segment in status requested.
pub fn new_adaptive_forecast_segment(code: &str, title: &str) -> AdaptiveForecastSegment {
    AdaptiveForecastSegment {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        host: "".to_string(),
        window_days: 0,
        active: false,
        archived: false,
        tags: Vec::new(),
        status: AdaptiveForecastSegmentStatus::Requested,
    }
}
