use super::*;

use acme_kit::text;

/// Where a overage entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl OverageEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for OverageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage entry as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageEntry {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub pinned: bool,
    pub port: i64,
    pub status: OverageEntryStatus,
}

impl OverageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageEntryStatus) -> bool {
        match self.status {
            OverageEntryStatus::Proposed => {
                return next == OverageEntryStatus::Approved || next == OverageEntryStatus::Withdrawn;
            }
            OverageEntryStatus::Approved => {
                match next {
                    OverageEntryStatus::Rejected | OverageEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageEntryStatus::Rejected => {
                match next {
                    OverageEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: OverageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> OverageEntry {
        OverageEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            pinned: self.pinned,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a overage entry in status proposed.
pub fn new_overage_entry(code: &str, name: &str) -> OverageEntry {
    OverageEntry {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        pinned: false,
        port: 0,
        status: OverageEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_overage_entry("code-131", "name-176");
        assert_eq!(item.status, OverageEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_overage_entry("code-131", "name-176");
        if item.move_to(OverageEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_overage_entry("code-131", "name-176");
        item.status = OverageEntryStatus::Withdrawn;
        assert!(
            item.move_to(OverageEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
