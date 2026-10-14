use super::*;

use acme_kit::text;

/// Where a delegation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DelegationEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DelegationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationEntry {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub ends_at: i64,
    pub audience: String,
    pub priority: i64,
    pub ratio: f64,
    pub discount: f64,
    pub quantity: i64,
    pub ttl_seconds: i64,
    pub status: DelegationEntryStatus,
}

impl DelegationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationEntryStatus) -> bool {
        match self.status {
            DelegationEntryStatus::Requested => {
                return next == DelegationEntryStatus::Revoked;
            }
            DelegationEntryStatus::Granted => {
                return next == DelegationEntryStatus::Revoked || next == DelegationEntryStatus::Expired;
            }
            DelegationEntryStatus::Revoked => {
                match next {
                    DelegationEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: DelegationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation entry {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.discount - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationEntry {
        DelegationEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            ends_at: self.ends_at,
            audience: self.audience.clone(),
            priority: self.priority,
            ratio: self.ratio,
            discount: self.discount,
            quantity: self.quantity,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation entry in status requested.
pub fn new_delegation_entry(code: &str, label: &str) -> DelegationEntry {
    DelegationEntry {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        ends_at: 0,
        audience: "".to_string(),
        priority: 0,
        ratio: 0.0,
        discount: 0.0,
        quantity: 0,
        ttl_seconds: 0,
        status: DelegationEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_delegation_entry("code-684", "label-926");
        assert_eq!(item.status, DelegationEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_delegation_entry("code-684", "label-926");
        if item.move_to(DelegationEntryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationEntryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_delegation_entry("code-684", "label-926");
        item.status = DelegationEntryStatus::Expired;
        assert!(
            item.move_to(DelegationEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
