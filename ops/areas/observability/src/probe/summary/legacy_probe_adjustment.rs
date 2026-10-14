use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy probe adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyProbeAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyProbeAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy probe adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAdjustment {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub quantity: i64,
    pub maximum: f64,
    pub attributes: BTreeMap<String, String>,
    pub segment: String,
    pub revision: i64,
    pub label: String,
    pub archived: bool,
    pub ratio: f64,
    pub status: LegacyProbeAdjustmentStatus,
}

impl LegacyProbeAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeAdjustmentStatus) -> bool {
        match self.status {
            LegacyProbeAdjustmentStatus::Proposed => {
                return next == LegacyProbeAdjustmentStatus::Rejected || next == LegacyProbeAdjustmentStatus::Withdrawn;
            }
            LegacyProbeAdjustmentStatus::Approved => {
                return next == LegacyProbeAdjustmentStatus::Withdrawn;
            }
            LegacyProbeAdjustmentStatus::Rejected => {
                return next == LegacyProbeAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyProbeAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyProbeAdjustment {
        LegacyProbeAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            quantity: self.quantity,
            maximum: self.maximum,
            attributes: self.attributes.clone(),
            segment: self.segment.clone(),
            revision: self.revision,
            label: self.label.clone(),
            archived: self.archived,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy probe adjustment in status proposed.
pub fn new_legacy_probe_adjustment(code: &str, name: &str) -> LegacyProbeAdjustment {
    LegacyProbeAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        quantity: 0,
        maximum: 0.0,
        attributes: BTreeMap::new(),
        segment: "".to_string(),
        revision: 0,
        label: "".to_string(),
        archived: false,
        ratio: 0.0,
        status: LegacyProbeAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_probe_adjustment("code-192", "name-956");
        assert_eq!(item.status, LegacyProbeAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_legacy_probe_adjustment("code-192", "name-956");
        if item.move_to(LegacyProbeAdjustmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbeAdjustmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_probe_adjustment("code-192", "name-956");
        item.status = LegacyProbeAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(LegacyProbeAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
