use super::*;

use acme_kit::text;

/// Where a region pin draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionPinDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RegionPinDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDraft {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub checksum: String,
    pub weight: f64,
    pub window_days: i64,
    pub grace_days: i64,
    pub status: RegionPinDraftStatus,
}

impl RegionPinDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinDraftStatus) -> bool {
        match self.status {
            RegionPinDraftStatus::Draft => {
                return next == RegionPinDraftStatus::Suspended;
            }
            RegionPinDraftStatus::Pending => {
                return next == RegionPinDraftStatus::Active;
            }
            RegionPinDraftStatus::Active => {
                match next {
                    RegionPinDraftStatus::Suspended | RegionPinDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinDraftStatus::Suspended => {
                match next {
                    RegionPinDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: RegionPinDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.weight + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinDraft {
        RegionPinDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            checksum: self.checksum.clone(),
            weight: self.weight,
            window_days: self.window_days,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a region pin draft in status draft.
pub fn new_region_pin_draft(code: &str, name: &str) -> RegionPinDraft {
    RegionPinDraft {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        checksum: "".to_string(),
        weight: 0.0,
        window_days: 0,
        grace_days: 0,
        status: RegionPinDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_region_pin_draft("code-612", "name-780");
        assert_eq!(item.status, RegionPinDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_region_pin_draft("code-612", "name-780");
        if item.move_to(RegionPinDraftStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinDraftStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_region_pin_draft("code-612", "name-780");
        item.status = RegionPinDraftStatus::Closed;
        assert!(
            item.move_to(RegionPinDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
