use super::*;

use acme_kit::text;

/// Where a slo entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloEntryStatus {
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

impl std::fmt::Display for SloEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloEntry {
    pub code: String,
    pub name: String,
    pub source: String,
    pub timezone: String,
    pub rate: f64,
    pub status: SloEntryStatus,
}

impl SloEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloEntryStatus) -> bool {
        match self.status {
            SloEntryStatus::Open => {
                return next == SloEntryStatus::Resolved;
            }
            SloEntryStatus::InReview => {
                match next {
                    SloEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloEntryStatus::Resolved => {
                return next == SloEntryStatus::Archived;
            }
            SloEntryStatus::Reopened => {
                match next {
                    SloEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: SloEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo entry {} cannot move from {} to {}",
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
        self.rate
    }

    pub fn copy(&self) -> SloEntry {
        SloEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            timezone: self.timezone.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a slo entry in status open.
pub fn new_slo_entry(code: &str, name: &str) -> SloEntry {
    SloEntry {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        timezone: "".to_string(),
        rate: 0.0,
        status: SloEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_slo_entry("code-456", "name-303");
        assert_eq!(item.status, SloEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_slo_entry("code-456", "name-303");
        if item.move_to(SloEntryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloEntryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_slo_entry("code-456", "name-303");
        item.status = SloEntryStatus::Archived;
        assert!(item.move_to(SloEntryStatus::Open).is_err(), "expected move to fail");
    }
}
