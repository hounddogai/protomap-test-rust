use super::*;

use acme_kit::text;

/// Where a region pin baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionPinBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RegionPinBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaseline {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub tags: Vec<String>,
    pub sequence: i64,
    pub ttl_seconds: i64,
    pub status: RegionPinBaselineStatus,
}

impl RegionPinBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinBaselineStatus) -> bool {
        match self.status {
            RegionPinBaselineStatus::Planned => {
                match next {
                    RegionPinBaselineStatus::InProgress | RegionPinBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinBaselineStatus::Scheduled => {
                match next {
                    RegionPinBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinBaselineStatus::InProgress => {
                return next == RegionPinBaselineStatus::Completed || next == RegionPinBaselineStatus::Cancelled;
            }
            RegionPinBaselineStatus::Completed => {
                match next {
                    RegionPinBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionPinBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.sequence as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinBaseline {
        RegionPinBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            tags: self.tags.clone(),
            sequence: self.sequence,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin baseline in status planned.
pub fn new_region_pin_baseline(code: &str, label: &str) -> RegionPinBaseline {
    RegionPinBaseline {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        tags: Vec::new(),
        sequence: 0,
        ttl_seconds: 0,
        status: RegionPinBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_region_pin_baseline("code-333", "label-559");
        assert_eq!(item.status, RegionPinBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_region_pin_baseline("code-333", "label-559");
        if item.move_to(RegionPinBaselineStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinBaselineStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_region_pin_baseline("code-333", "label-559");
        item.status = RegionPinBaselineStatus::Cancelled;
        assert!(
            item.move_to(RegionPinBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
