use super::*;

use acme_kit::text;

/// Where a failover entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEntry {
    pub code: String,
    pub name: String,
    pub host: String,
    pub scope: String,
    pub label: String,
    pub phone: String,
    pub locale: String,
    pub reviewer: String,
    pub rotation_days: i64,
    pub status: FailoverEntryStatus,
}

impl FailoverEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverEntryStatus) -> bool {
        match self.status {
            FailoverEntryStatus::Proposed => {
                return next == FailoverEntryStatus::Rejected;
            }
            FailoverEntryStatus::Approved => {
                return next == FailoverEntryStatus::Rejected || next == FailoverEntryStatus::Withdrawn;
            }
            FailoverEntryStatus::Rejected => {
                match next {
                    FailoverEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FailoverEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> FailoverEntry {
        FailoverEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            scope: self.scope.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a failover entry in status proposed.
pub fn new_failover_entry(code: &str, name: &str) -> FailoverEntry {
    FailoverEntry {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        scope: "".to_string(),
        label: "".to_string(),
        phone: "".to_string(),
        locale: "".to_string(),
        reviewer: "".to_string(),
        rotation_days: 0,
        status: FailoverEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_failover_entry("code-805", "name-96");
        assert_eq!(item.status, FailoverEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_failover_entry("code-805", "name-96");
        if item.move_to(FailoverEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_failover_entry("code-805", "name-96");
        item.status = FailoverEntryStatus::Withdrawn;
        assert!(
            item.move_to(FailoverEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
