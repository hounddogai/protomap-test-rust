use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched segment forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSegmentForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedSegmentForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedSegmentForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched segment forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentForecast {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub pinned: bool,
    pub sequence: i64,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: BatchedSegmentForecastStatus,
}

impl BatchedSegmentForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSegmentForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSegmentForecastStatus) -> bool {
        match self.status {
            BatchedSegmentForecastStatus::Requested => {
                match next {
                    BatchedSegmentForecastStatus::Revoked | BatchedSegmentForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentForecastStatus::Granted => {
                return next == BatchedSegmentForecastStatus::Revoked || next == BatchedSegmentForecastStatus::Expired;
            }
            BatchedSegmentForecastStatus::Revoked => {
                return next == BatchedSegmentForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSegmentForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched segment forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched segment forecast", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSegmentForecast {
        BatchedSegmentForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            sequence: self.sequence,
            rate: self.rate,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched segment forecast in status requested.
pub fn new_batched_segment_forecast(code: &str, label: &str) -> BatchedSegmentForecast {
    BatchedSegmentForecast {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        pinned: false,
        sequence: 0,
        rate: 0.0,
        attributes: BTreeMap::new(),
        status: BatchedSegmentForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_segment_forecast("code-574", "label-711");
        assert_eq!(item.status, BatchedSegmentForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_batched_segment_forecast("code-574", "label-711");
        if item.move_to(BatchedSegmentForecastStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSegmentForecastStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_segment_forecast("code-574", "label-711");
        item.status = BatchedSegmentForecastStatus::Expired;
        assert!(
            item.move_to(BatchedSegmentForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
