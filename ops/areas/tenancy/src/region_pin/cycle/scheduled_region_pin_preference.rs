use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled region pin preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRegionPinPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledRegionPinPreferenceStatus {
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

impl std::fmt::Display for ScheduledRegionPinPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled region pin preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinPreference {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub budget: f64,
    pub approved: bool,
    pub segment: String,
    pub summary: String,
    pub owner_id: String,
    pub limit: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: ScheduledRegionPinPreferenceStatus,
}

impl ScheduledRegionPinPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRegionPinPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRegionPinPreferenceStatus) -> bool {
        match self.status {
            ScheduledRegionPinPreferenceStatus::Idle => {
                return next == ScheduledRegionPinPreferenceStatus::Ready;
            }
            ScheduledRegionPinPreferenceStatus::Warming => {
                return next == ScheduledRegionPinPreferenceStatus::Ready
                    || next == ScheduledRegionPinPreferenceStatus::Stopped;
            }
            ScheduledRegionPinPreferenceStatus::Ready => {
                match next {
                    ScheduledRegionPinPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRegionPinPreferenceStatus::Draining => {
                return next == ScheduledRegionPinPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRegionPinPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled region pin preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.budget - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRegionPinPreference {
        ScheduledRegionPinPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            budget: self.budget,
            approved: self.approved,
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            owner_id: self.owner_id.clone(),
            limit: self.limit,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled region pin preference in status idle.
pub fn new_scheduled_region_pin_preference(code: &str, title: &str) -> ScheduledRegionPinPreference {
    ScheduledRegionPinPreference {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        budget: 0.0,
        approved: false,
        segment: "".to_string(),
        summary: "".to_string(),
        owner_id: "".to_string(),
        limit: 0,
        attributes: BTreeMap::new(),
        status: ScheduledRegionPinPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_region_pin_preference("code-620", "title-60");
        assert_eq!(item.status, ScheduledRegionPinPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_region_pin_preference("code-620", "title-60");
        if item.move_to(ScheduledRegionPinPreferenceStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRegionPinPreferenceStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_region_pin_preference("code-620", "title-60");
        item.status = ScheduledRegionPinPreferenceStatus::Stopped;
        assert!(
            item.move_to(ScheduledRegionPinPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
