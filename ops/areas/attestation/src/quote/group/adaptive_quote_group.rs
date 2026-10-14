use super::*;

use acme_kit::text;

/// Where a adaptive quote group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveQuoteGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveQuoteGroupStatus {
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

impl std::fmt::Display for AdaptiveQuoteGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive quote group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroup {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub cluster: String,
    pub threshold: f64,
    pub country: String,
    pub weight: f64,
    pub rate: f64,
    pub status: AdaptiveQuoteGroupStatus,
}

impl AdaptiveQuoteGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveQuoteGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveQuoteGroupStatus) -> bool {
        match self.status {
            AdaptiveQuoteGroupStatus::Idle => {
                return next == AdaptiveQuoteGroupStatus::Ready || next == AdaptiveQuoteGroupStatus::Stopped;
            }
            AdaptiveQuoteGroupStatus::Warming => {
                return next == AdaptiveQuoteGroupStatus::Stopped;
            }
            AdaptiveQuoteGroupStatus::Ready => {
                return next == AdaptiveQuoteGroupStatus::Draining;
            }
            AdaptiveQuoteGroupStatus::Draining => {
                return next == AdaptiveQuoteGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveQuoteGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive quote group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive quote group", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.rate + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveQuoteGroup {
        AdaptiveQuoteGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            cluster: self.cluster.clone(),
            threshold: self.threshold,
            country: self.country.clone(),
            weight: self.weight,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a adaptive quote group in status idle.
pub fn new_adaptive_quote_group(code: &str, label: &str) -> AdaptiveQuoteGroup {
    AdaptiveQuoteGroup {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        cluster: "".to_string(),
        threshold: 0.0,
        country: "".to_string(),
        weight: 0.0,
        rate: 0.0,
        status: AdaptiveQuoteGroupStatus::Idle,
    }
}
