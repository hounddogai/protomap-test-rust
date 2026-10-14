use super::*;

use acme_kit::text;

/// Where a capped slo entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSloEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedSloEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedSloEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped slo entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntry {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub reviewer: String,
    pub pinned: bool,
    pub key_id: String,
    pub attempts: i64,
    pub status: CappedSloEntryStatus,
}

impl CappedSloEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSloEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSloEntryStatus) -> bool {
        match self.status {
            CappedSloEntryStatus::Requested => {
                return next == CappedSloEntryStatus::Granted || next == CappedSloEntryStatus::Expired;
            }
            CappedSloEntryStatus::Granted => {
                return next == CappedSloEntryStatus::Revoked;
            }
            CappedSloEntryStatus::Revoked => {
                return next == CappedSloEntryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSloEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped slo entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.attempts as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSloEntry {
        CappedSloEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            reviewer: self.reviewer.clone(),
            pinned: self.pinned,
            key_id: self.key_id.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped slo entry in status requested.
pub fn new_capped_slo_entry(code: &str, label: &str) -> CappedSloEntry {
    CappedSloEntry {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        reviewer: "".to_string(),
        pinned: false,
        key_id: "".to_string(),
        attempts: 0,
        status: CappedSloEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_slo_entry("code-994", "label-327");
        assert_eq!(item.status, CappedSloEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_slo_entry("code-994", "label-327");
        if item.move_to(CappedSloEntryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSloEntryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_slo_entry("code-994", "label-327");
        item.status = CappedSloEntryStatus::Expired;
        assert!(
            item.move_to(CappedSloEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
