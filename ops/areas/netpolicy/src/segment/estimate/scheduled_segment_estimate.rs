use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled segment estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSegmentEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledSegmentEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledSegmentEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled segment estimate as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEstimate {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub attributes: BTreeMap<String, String>,
    pub notes: String,
    pub window_days: i64,
    pub principal: String,
    pub floor_value: f64,
    pub visible: bool,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub status: ScheduledSegmentEstimateStatus,
}

impl ScheduledSegmentEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSegmentEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSegmentEstimateStatus) -> bool {
        match self.status {
            ScheduledSegmentEstimateStatus::Proposed => {
                return next == ScheduledSegmentEstimateStatus::Withdrawn;
            }
            ScheduledSegmentEstimateStatus::Approved => {
                match next {
                    ScheduledSegmentEstimateStatus::Rejected | ScheduledSegmentEstimateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentEstimateStatus::Rejected => {
                match next {
                    ScheduledSegmentEstimateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledSegmentEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled segment estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled segment estimate", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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
        ((self.starts_at as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSegmentEstimate {
        ScheduledSegmentEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            attributes: self.attributes.clone(),
            notes: self.notes.clone(),
            window_days: self.window_days,
            principal: self.principal.clone(),
            floor_value: self.floor_value,
            visible: self.visible,
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled segment estimate in status proposed.
pub fn new_scheduled_segment_estimate(code: &str, name: &str) -> ScheduledSegmentEstimate {
    ScheduledSegmentEstimate {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        attributes: BTreeMap::new(),
        notes: "".to_string(),
        window_days: 0,
        principal: "".to_string(),
        floor_value: 0.0,
        visible: false,
        starts_at: 0,
        tags: Vec::new(),
        status: ScheduledSegmentEstimateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_segment_estimate("code-539", "name-561");
        assert_eq!(item.status, ScheduledSegmentEstimateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_scheduled_segment_estimate("code-539", "name-561");
        if item.move_to(ScheduledSegmentEstimateStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSegmentEstimateStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_segment_estimate("code-539", "name-561");
        item.status = ScheduledSegmentEstimateStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledSegmentEstimateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
