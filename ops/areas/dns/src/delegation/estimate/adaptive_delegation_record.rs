use super::*;

use acme_kit::text;

/// Where a adaptive delegation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDelegationRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveDelegationRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveDelegationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive delegation record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecord {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub maximum: f64,
    pub label: String,
    pub approved: bool,
    pub country: String,
    pub key_id: String,
    pub ends_at: i64,
    pub notes: String,
    pub cluster: String,
    pub status: AdaptiveDelegationRecordStatus,
}

impl AdaptiveDelegationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDelegationRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDelegationRecordStatus) -> bool {
        match self.status {
            AdaptiveDelegationRecordStatus::Proposed => {
                return next == AdaptiveDelegationRecordStatus::Approved
                    || next == AdaptiveDelegationRecordStatus::Rejected;
            }
            AdaptiveDelegationRecordStatus::Approved => {
                return next == AdaptiveDelegationRecordStatus::Rejected
                    || next == AdaptiveDelegationRecordStatus::Withdrawn;
            }
            AdaptiveDelegationRecordStatus::Rejected => {
                return next == AdaptiveDelegationRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveDelegationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive delegation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive delegation record", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.maximum + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveDelegationRecord {
        AdaptiveDelegationRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            maximum: self.maximum,
            label: self.label.clone(),
            approved: self.approved,
            country: self.country.clone(),
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            notes: self.notes.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive delegation record in status proposed.
pub fn new_adaptive_delegation_record(code: &str, name: &str) -> AdaptiveDelegationRecord {
    AdaptiveDelegationRecord {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        maximum: 0.0,
        label: "".to_string(),
        approved: false,
        country: "".to_string(),
        key_id: "".to_string(),
        ends_at: 0,
        notes: "".to_string(),
        cluster: "".to_string(),
        status: AdaptiveDelegationRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_delegation_record("code-774", "name-437");
        assert_eq!(item.status, AdaptiveDelegationRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_adaptive_delegation_record("code-774", "name-437");
        if item.move_to(AdaptiveDelegationRecordStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveDelegationRecordStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_delegation_record("code-774", "name-437");
        item.status = AdaptiveDelegationRecordStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveDelegationRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
