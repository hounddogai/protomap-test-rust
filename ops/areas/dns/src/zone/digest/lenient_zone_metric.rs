use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient zone metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientZoneMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientZoneMetricStatus {
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

impl std::fmt::Display for LenientZoneMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient zone metric as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneMetric {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub notes: String,
    pub checksum: String,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub status: LenientZoneMetricStatus,
}

impl LenientZoneMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientZoneMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientZoneMetricStatus) -> bool {
        match self.status {
            LenientZoneMetricStatus::Idle => {
                return next == LenientZoneMetricStatus::Stopped;
            }
            LenientZoneMetricStatus::Warming => {
                match next {
                    LenientZoneMetricStatus::Draining | LenientZoneMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientZoneMetricStatus::Ready => {
                return next == LenientZoneMetricStatus::Stopped;
            }
            LenientZoneMetricStatus::Draining => {
                match next {
                    LenientZoneMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientZoneMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient zone metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientZoneMetric {
        LenientZoneMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient zone metric in status idle.
pub fn new_lenient_zone_metric(code: &str, label: &str) -> LenientZoneMetric {
    LenientZoneMetric {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        notes: "".to_string(),
        checksum: "".to_string(),
        attributes: BTreeMap::new(),
        updated_at: 0,
        status: LenientZoneMetricStatus::Idle,
    }
}
