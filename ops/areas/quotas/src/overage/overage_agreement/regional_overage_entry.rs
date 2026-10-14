use super::*;

use acme_kit::text;

/// Where a regional overage entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalOverageEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalOverageEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalOverageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional overage entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntry {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub summary: String,
    pub reviewer: String,
    pub minimum: f64,
    pub audience: String,
    pub status: RegionalOverageEntryStatus,
}

impl RegionalOverageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalOverageEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalOverageEntryStatus) -> bool {
        match self.status {
            RegionalOverageEntryStatus::Requested => {
                return next == RegionalOverageEntryStatus::Granted || next == RegionalOverageEntryStatus::Revoked;
            }
            RegionalOverageEntryStatus::Granted => {
                match next {
                    RegionalOverageEntryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalOverageEntryStatus::Revoked => {
                match next {
                    RegionalOverageEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalOverageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional overage entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional overage entry", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.minimum + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalOverageEntry {
        RegionalOverageEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            summary: self.summary.clone(),
            reviewer: self.reviewer.clone(),
            minimum: self.minimum,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional overage entry in status requested.
pub fn new_regional_overage_entry(code: &str, title: &str) -> RegionalOverageEntry {
    RegionalOverageEntry {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        summary: "".to_string(),
        reviewer: "".to_string(),
        minimum: 0.0,
        audience: "".to_string(),
        status: RegionalOverageEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_overage_entry("code-367", "title-449");
        assert_eq!(item.status, RegionalOverageEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_overage_entry("code-367", "title-449");
        if item.move_to(RegionalOverageEntryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalOverageEntryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_overage_entry("code-367", "title-449");
        item.status = RegionalOverageEntryStatus::Expired;
        assert!(
            item.move_to(RegionalOverageEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
