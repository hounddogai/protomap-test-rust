use super::*;

use acme_kit::text;

/// Where a release entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReleaseEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReleaseEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEntry {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub version: i64,
    pub visible: bool,
    pub owner_id: String,
    pub host: String,
    pub key_id: String,
    pub status: ReleaseEntryStatus,
}

impl ReleaseEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseEntryStatus) -> bool {
        match self.status {
            ReleaseEntryStatus::Proposed => {
                match next {
                    ReleaseEntryStatus::Approved | ReleaseEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseEntryStatus::Approved => {
                match next {
                    ReleaseEntryStatus::Rejected | ReleaseEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseEntryStatus::Rejected => {
                return next == ReleaseEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> ReleaseEntry {
        ReleaseEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            version: self.version,
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            host: self.host.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a release entry in status proposed.
pub fn new_release_entry(code: &str, title: &str) -> ReleaseEntry {
    ReleaseEntry {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        version: 0,
        visible: false,
        owner_id: "".to_string(),
        host: "".to_string(),
        key_id: "".to_string(),
        status: ReleaseEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_release_entry("code-298", "title-651");
        assert_eq!(item.status, ReleaseEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_release_entry("code-298", "title-651");
        if item.move_to(ReleaseEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_release_entry("code-298", "title-651");
        item.status = ReleaseEntryStatus::Withdrawn;
        assert!(
            item.move_to(ReleaseEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
