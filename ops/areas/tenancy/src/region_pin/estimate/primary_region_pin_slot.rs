use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary region pin slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRegionPinSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryRegionPinSlotStatus {
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

impl std::fmt::Display for PrimaryRegionPinSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary region pin slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlot {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub visible: bool,
    pub status: PrimaryRegionPinSlotStatus,
}

impl PrimaryRegionPinSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRegionPinSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRegionPinSlotStatus) -> bool {
        match self.status {
            PrimaryRegionPinSlotStatus::Planned => {
                return next == PrimaryRegionPinSlotStatus::InProgress || next == PrimaryRegionPinSlotStatus::Completed;
            }
            PrimaryRegionPinSlotStatus::Scheduled => {
                return next == PrimaryRegionPinSlotStatus::Cancelled;
            }
            PrimaryRegionPinSlotStatus::InProgress => {
                match next {
                    PrimaryRegionPinSlotStatus::Completed | PrimaryRegionPinSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRegionPinSlotStatus::Completed => {
                match next {
                    PrimaryRegionPinSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryRegionPinSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary region pin slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary region pin slot", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> PrimaryRegionPinSlot {
        PrimaryRegionPinSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary region pin slot in status planned.
pub fn new_primary_region_pin_slot(code: &str, label: &str) -> PrimaryRegionPinSlot {
    PrimaryRegionPinSlot {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        threshold: 0.0,
        visible: false,
        status: PrimaryRegionPinSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_region_pin_slot("code-275", "label-433");
        assert_eq!(item.status, PrimaryRegionPinSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_primary_region_pin_slot("code-275", "label-433");
        if item.move_to(PrimaryRegionPinSlotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRegionPinSlotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_region_pin_slot("code-275", "label-433");
        item.status = PrimaryRegionPinSlotStatus::Cancelled;
        assert!(
            item.move_to(PrimaryRegionPinSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
