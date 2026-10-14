use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary redaction metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRedactionMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryRedactionMetricStatus {
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

impl std::fmt::Display for PrimaryRedactionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary redaction metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionMetric {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub cluster: String,
    pub capacity: i64,
    pub notes: String,
    pub attributes: BTreeMap<String, String>,
    pub key_id: String,
    pub discount: f64,
    pub status: PrimaryRedactionMetricStatus,
}

impl PrimaryRedactionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRedactionMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRedactionMetricStatus) -> bool {
        match self.status {
            PrimaryRedactionMetricStatus::Idle => {
                match next {
                    PrimaryRedactionMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRedactionMetricStatus::Warming => {
                return next == PrimaryRedactionMetricStatus::Stopped;
            }
            PrimaryRedactionMetricStatus::Ready => {
                return next == PrimaryRedactionMetricStatus::Draining || next == PrimaryRedactionMetricStatus::Stopped;
            }
            PrimaryRedactionMetricStatus::Draining => {
                match next {
                    PrimaryRedactionMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryRedactionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary redaction metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary redaction metric", self.code)
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

    pub fn projected_exposure(&self) -> f64 {
        ((self.limit as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRedactionMetric {
        PrimaryRedactionMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            cluster: self.cluster.clone(),
            capacity: self.capacity,
            notes: self.notes.clone(),
            attributes: self.attributes.clone(),
            key_id: self.key_id.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary redaction metric in status idle.
pub fn new_primary_redaction_metric(code: &str, title: &str) -> PrimaryRedactionMetric {
    PrimaryRedactionMetric {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        cluster: "".to_string(),
        capacity: 0,
        notes: "".to_string(),
        attributes: BTreeMap::new(),
        key_id: "".to_string(),
        discount: 0.0,
        status: PrimaryRedactionMetricStatus::Idle,
    }
}
