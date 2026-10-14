use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanSegmentStatus {
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

impl std::fmt::Display for PlanSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegment {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub key_id: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub active: bool,
    pub threshold: f64,
    pub status: PlanSegmentStatus,
}

impl PlanSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanSegmentStatus) -> bool {
        match self.status {
            PlanSegmentStatus::Idle => {
                return next == PlanSegmentStatus::Warming || next == PlanSegmentStatus::Ready;
            }
            PlanSegmentStatus::Warming => {
                match next {
                    PlanSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanSegmentStatus::Ready => {
                match next {
                    PlanSegmentStatus::Draining | PlanSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanSegmentStatus::Draining => {
                match next {
                    PlanSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan segment", self.code)
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

    pub fn projected_load(&self) -> f64 {
        ((self.threshold + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanSegment {
        PlanSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            active: self.active,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan segment in status idle.
pub fn new_plan_segment(code: &str, name: &str) -> PlanSegment {
    PlanSegment {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        key_id: "".to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        active: false,
        threshold: 0.0,
        status: PlanSegmentStatus::Idle,
    }
}
