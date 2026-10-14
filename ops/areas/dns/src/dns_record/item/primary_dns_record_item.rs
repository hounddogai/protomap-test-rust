use super::*;

use acme_kit::text;

/// Where a primary dns record item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDnsRecordItemStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryDnsRecordItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryDnsRecordItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary dns record item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItem {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub scope: String,
    pub rate: f64,
    pub checksum: String,
    pub reviewer: String,
    pub grace_days: i64,
    pub fingerprint: String,
    pub expires_at: i64,
    pub pinned: bool,
    pub status: PrimaryDnsRecordItemStatus,
}

impl PrimaryDnsRecordItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDnsRecordItemStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDnsRecordItemStatus) -> bool {
        match self.status {
            PrimaryDnsRecordItemStatus::Proposed => {
                match next {
                    PrimaryDnsRecordItemStatus::Rejected | PrimaryDnsRecordItemStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDnsRecordItemStatus::Approved => {
                return next == PrimaryDnsRecordItemStatus::Rejected;
            }
            PrimaryDnsRecordItemStatus::Rejected => {
                return next == PrimaryDnsRecordItemStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDnsRecordItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary dns record item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.expires_at as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDnsRecordItem {
        PrimaryDnsRecordItem {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            scope: self.scope.clone(),
            rate: self.rate,
            checksum: self.checksum.clone(),
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            fingerprint: self.fingerprint.clone(),
            expires_at: self.expires_at,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary dns record item in status proposed.
pub fn new_primary_dns_record_item(code: &str, name: &str) -> PrimaryDnsRecordItem {
    PrimaryDnsRecordItem {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        scope: "".to_string(),
        rate: 0.0,
        checksum: "".to_string(),
        reviewer: "".to_string(),
        grace_days: 0,
        fingerprint: "".to_string(),
        expires_at: 0,
        pinned: false,
        status: PrimaryDnsRecordItemStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_dns_record_item("code-366", "name-659");
        assert_eq!(item.status, PrimaryDnsRecordItemStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_primary_dns_record_item("code-366", "name-659");
        if item.move_to(PrimaryDnsRecordItemStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryDnsRecordItemStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_dns_record_item("code-366", "name-659");
        item.status = PrimaryDnsRecordItemStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryDnsRecordItemStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
