use super::*;

use acme_kit::text;

/// Where a strict page entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPageEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictPageEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictPageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict page entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageEntry {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub archived: bool,
    pub category: String,
    pub quantity: i64,
    pub status: StrictPageEntryStatus,
}

impl StrictPageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPageEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPageEntryStatus) -> bool {
        match self.status {
            StrictPageEntryStatus::Requested => {
                return next == StrictPageEntryStatus::Revoked || next == StrictPageEntryStatus::Expired;
            }
            StrictPageEntryStatus::Granted => {
                return next == StrictPageEntryStatus::Revoked || next == StrictPageEntryStatus::Expired;
            }
            StrictPageEntryStatus::Revoked => {
                match next {
                    StrictPageEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: StrictPageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict page entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> StrictPageEntry {
        StrictPageEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            archived: self.archived,
            category: self.category.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict page entry in status requested.
pub fn new_strict_page_entry(code: &str, name: &str) -> StrictPageEntry {
    StrictPageEntry {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        archived: false,
        category: "".to_string(),
        quantity: 0,
        status: StrictPageEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_page_entry("code-173", "name-414");
        assert_eq!(item.status, StrictPageEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_strict_page_entry("code-173", "name-414");
        if item.move_to(StrictPageEntryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPageEntryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_page_entry("code-173", "name-414");
        item.status = StrictPageEntryStatus::Expired;
        assert!(
            item.move_to(StrictPageEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
