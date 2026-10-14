use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a slo target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SloTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SloTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo target as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloTarget {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub quota_units: i64,
    pub fingerprint: String,
    pub ceiling: f64,
    pub attributes: BTreeMap<String, String>,
    pub channel: String,
    pub reviewer: String,
    pub source: String,
    pub status: SloTargetStatus,
}

impl SloTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloTargetStatus) -> bool {
        match self.status {
            SloTargetStatus::Proposed => {
                return next == SloTargetStatus::Rejected;
            }
            SloTargetStatus::Approved => {
                return next == SloTargetStatus::Rejected || next == SloTargetStatus::Withdrawn;
            }
            SloTargetStatus::Rejected => {
                match next {
                    SloTargetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SloTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo target", self.code)
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

    pub fn weighted_load(&self) -> f64 {
        ((self.ceiling + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloTarget {
        SloTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            quota_units: self.quota_units,
            fingerprint: self.fingerprint.clone(),
            ceiling: self.ceiling,
            attributes: self.attributes.clone(),
            channel: self.channel.clone(),
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a slo target in status proposed.
pub fn new_slo_target(code: &str, label: &str) -> SloTarget {
    SloTarget {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        quota_units: 0,
        fingerprint: "".to_string(),
        ceiling: 0.0,
        attributes: BTreeMap::new(),
        channel: "".to_string(),
        reviewer: "".to_string(),
        source: "".to_string(),
        status: SloTargetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_slo_target("code-488", "label-870");
        assert_eq!(item.status, SloTargetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_slo_target("code-488", "label-870");
        if item.move_to(SloTargetStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloTargetStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_slo_target("code-488", "label-870");
        item.status = SloTargetStatus::Withdrawn;
        assert!(
            item.move_to(SloTargetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
