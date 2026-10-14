use super::*;

use acme_kit::text;

/// Where a capped dns record entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDnsRecordEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedDnsRecordEntryStatus {
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

impl std::fmt::Display for CappedDnsRecordEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped dns record entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDnsRecordEntry {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub title: String,
    pub key_id: String,
    pub category: String,
    pub visible: bool,
    pub grace_days: i64,
    pub status: CappedDnsRecordEntryStatus,
}

impl CappedDnsRecordEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDnsRecordEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDnsRecordEntryStatus) -> bool {
        match self.status {
            CappedDnsRecordEntryStatus::Draft => {
                return next == CappedDnsRecordEntryStatus::Active || next == CappedDnsRecordEntryStatus::Suspended;
            }
            CappedDnsRecordEntryStatus::Pending => {
                match next {
                    CappedDnsRecordEntryStatus::Active | CappedDnsRecordEntryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDnsRecordEntryStatus::Active => {
                match next {
                    CappedDnsRecordEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDnsRecordEntryStatus::Suspended => {
                return next == CappedDnsRecordEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedDnsRecordEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped dns record entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDnsRecordEntry {
        CappedDnsRecordEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            category: self.category.clone(),
            visible: self.visible,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped dns record entry in status draft.
pub fn new_capped_dns_record_entry(code: &str, name: &str) -> CappedDnsRecordEntry {
    CappedDnsRecordEntry {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        title: "".to_string(),
        key_id: "".to_string(),
        category: "".to_string(),
        visible: false,
        grace_days: 0,
        status: CappedDnsRecordEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_dns_record_entry("code-560", "name-869");
        assert_eq!(item.status, CappedDnsRecordEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_capped_dns_record_entry("code-560", "name-869");
        if item.move_to(CappedDnsRecordEntryStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDnsRecordEntryStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_dns_record_entry("code-560", "name-869");
        item.status = CappedDnsRecordEntryStatus::Closed;
        assert!(
            item.move_to(CappedDnsRecordEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
