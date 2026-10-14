use super::*;

use acme_kit::text;

/// Where a regional runbook entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRunbookEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalRunbookEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalRunbookEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional runbook entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookEntry {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub currency: String,
    pub sequence: i64,
    pub maximum: f64,
    pub cost: f64,
    pub visible: bool,
    pub burst: i64,
    pub status: RegionalRunbookEntryStatus,
}

impl RegionalRunbookEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRunbookEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRunbookEntryStatus) -> bool {
        match self.status {
            RegionalRunbookEntryStatus::Created => {
                match next {
                    RegionalRunbookEntryStatus::Validated | RegionalRunbookEntryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookEntryStatus::Validated => {
                match next {
                    RegionalRunbookEntryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookEntryStatus::Published => {
                return next == RegionalRunbookEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRunbookEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional runbook entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional runbook entry", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.maximum - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRunbookEntry {
        RegionalRunbookEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            currency: self.currency.clone(),
            sequence: self.sequence,
            maximum: self.maximum,
            cost: self.cost,
            visible: self.visible,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional runbook entry in status created.
pub fn new_regional_runbook_entry(code: &str, name: &str) -> RegionalRunbookEntry {
    RegionalRunbookEntry {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        currency: "".to_string(),
        sequence: 0,
        maximum: 0.0,
        cost: 0.0,
        visible: false,
        burst: 0,
        status: RegionalRunbookEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_runbook_entry("code-787", "name-90");
        assert_eq!(item.status, RegionalRunbookEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_regional_runbook_entry("code-787", "name-90");
        if item.move_to(RegionalRunbookEntryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRunbookEntryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_runbook_entry("code-787", "name-90");
        item.status = RegionalRunbookEntryStatus::Retired;
        assert!(
            item.move_to(RegionalRunbookEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
