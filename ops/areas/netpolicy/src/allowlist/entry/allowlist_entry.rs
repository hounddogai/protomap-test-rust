use super::*;

use acme_kit::text;

/// Where a allowlist entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AllowlistEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AllowlistEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntry {
    pub code: String,
    pub name: String,
    pub country: String,
    pub minimum: f64,
    pub checksum: String,
    pub quantity: i64,
    pub timezone: String,
    pub maximum: f64,
    pub notes: String,
    pub status: AllowlistEntryStatus,
}

impl AllowlistEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistEntryStatus) -> bool {
        match self.status {
            AllowlistEntryStatus::Proposed => {
                match next {
                    AllowlistEntryStatus::Approved | AllowlistEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistEntryStatus::Approved => {
                match next {
                    AllowlistEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistEntryStatus::Rejected => {
                match next {
                    AllowlistEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AllowlistEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.maximum + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistEntry {
        AllowlistEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            minimum: self.minimum,
            checksum: self.checksum.clone(),
            quantity: self.quantity,
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a allowlist entry in status proposed.
pub fn new_allowlist_entry(code: &str, name: &str) -> AllowlistEntry {
    AllowlistEntry {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        minimum: 0.0,
        checksum: "".to_string(),
        quantity: 0,
        timezone: "".to_string(),
        maximum: 0.0,
        notes: "".to_string(),
        status: AllowlistEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_allowlist_entry("code-886", "name-730");
        assert_eq!(item.status, AllowlistEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_allowlist_entry("code-886", "name-730");
        if item.move_to(AllowlistEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_allowlist_entry("code-886", "name-730");
        item.status = AllowlistEntryStatus::Withdrawn;
        assert!(
            item.move_to(AllowlistEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
