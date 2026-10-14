use super::*;

use acme_kit::text;

/// Where a dns record entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DnsRecordEntryStatus {
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

impl std::fmt::Display for DnsRecordEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record entry as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordEntry {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub name: String,
    pub unit: String,
    pub status: DnsRecordEntryStatus,
}

impl DnsRecordEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordEntryStatus) -> bool {
        match self.status {
            DnsRecordEntryStatus::Draft => {
                match next {
                    DnsRecordEntryStatus::Active | DnsRecordEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordEntryStatus::Pending => {
                match next {
                    DnsRecordEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordEntryStatus::Active => {
                match next {
                    DnsRecordEntryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordEntryStatus::Suspended => {
                match next {
                    DnsRecordEntryStatus::Closed => {
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

    pub fn move_to(&mut self, next: DnsRecordEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> DnsRecordEntry {
        DnsRecordEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            name: self.name.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a dns record entry in status draft.
pub fn new_dns_record_entry(code: &str, title: &str) -> DnsRecordEntry {
    DnsRecordEntry {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        name: "".to_string(),
        unit: "".to_string(),
        status: DnsRecordEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dns_record_entry("code-839", "title-892");
        assert_eq!(item.status, DnsRecordEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_dns_record_entry("code-839", "title-892");
        if item.move_to(DnsRecordEntryStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordEntryStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dns_record_entry("code-839", "title-892");
        item.status = DnsRecordEntryStatus::Closed;
        assert!(
            item.move_to(DnsRecordEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
