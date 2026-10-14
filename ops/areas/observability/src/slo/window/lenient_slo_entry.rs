use super::*;

use acme_kit::text;

/// Where a lenient slo entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSloEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientSloEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientSloEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient slo entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloEntry {
    pub code: String,
    pub name: String,
    pub host: String,
    pub reviewer: String,
    pub source: String,
    pub label: String,
    pub amount: f64,
    pub status: LenientSloEntryStatus,
}

impl LenientSloEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSloEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSloEntryStatus) -> bool {
        match self.status {
            LenientSloEntryStatus::Open => {
                return next == LenientSloEntryStatus::Reopened;
            }
            LenientSloEntryStatus::InReview => {
                match next {
                    LenientSloEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloEntryStatus::Resolved => {
                match next {
                    LenientSloEntryStatus::Reopened | LenientSloEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloEntryStatus::Reopened => {
                match next {
                    LenientSloEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientSloEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient slo entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> LenientSloEntry {
        LenientSloEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            label: self.label.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient slo entry in status open.
pub fn new_lenient_slo_entry(code: &str, name: &str) -> LenientSloEntry {
    LenientSloEntry {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        reviewer: "".to_string(),
        source: "".to_string(),
        label: "".to_string(),
        amount: 0.0,
        status: LenientSloEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_slo_entry("code-422", "name-980");
        assert_eq!(item.status, LenientSloEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_lenient_slo_entry("code-422", "name-980");
        if item.move_to(LenientSloEntryStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSloEntryStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_slo_entry("code-422", "name-980");
        item.status = LenientSloEntryStatus::Archived;
        assert!(
            item.move_to(LenientSloEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
