use super::*;

use acme_kit::text;

/// Where a zone entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntry {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub unit: String,
    pub attempts: i64,
    pub ceiling: f64,
    pub status: ZoneEntryStatus,
}

impl ZoneEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneEntryStatus) -> bool {
        match self.status {
            ZoneEntryStatus::Proposed => {
                return next == ZoneEntryStatus::Rejected || next == ZoneEntryStatus::Withdrawn;
            }
            ZoneEntryStatus::Approved => {
                match next {
                    ZoneEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneEntryStatus::Rejected => {
                match next {
                    ZoneEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ZoneEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.attempts as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneEntry {
        ZoneEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            unit: self.unit.clone(),
            attempts: self.attempts,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a zone entry in status proposed.
pub fn new_zone_entry(code: &str, name: &str) -> ZoneEntry {
    ZoneEntry {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        unit: "".to_string(),
        attempts: 0,
        ceiling: 0.0,
        status: ZoneEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_entry("code-29", "name-589");
        assert_eq!(item.status, ZoneEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_zone_entry("code-29", "name-589");
        if item.move_to(ZoneEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_entry("code-29", "name-589");
        item.status = ZoneEntryStatus::Withdrawn;
        assert!(
            item.move_to(ZoneEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
