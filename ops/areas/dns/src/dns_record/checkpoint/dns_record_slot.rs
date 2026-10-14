use super::*;

use acme_kit::text;

/// Where a dns record slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DnsRecordSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DnsRecordSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlot {
    pub code: String,
    pub name: String,
    pub source: String,
    pub notes: String,
    pub tenant_id: String,
    pub host: String,
    pub quantity: i64,
    pub status: DnsRecordSlotStatus,
}

impl DnsRecordSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordSlotStatus) -> bool {
        match self.status {
            DnsRecordSlotStatus::Planned => {
                match next {
                    DnsRecordSlotStatus::Scheduled | DnsRecordSlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordSlotStatus::Scheduled => {
                return next == DnsRecordSlotStatus::Completed;
            }
            DnsRecordSlotStatus::InProgress => {
                return next == DnsRecordSlotStatus::Cancelled;
            }
            DnsRecordSlotStatus::Completed => {
                match next {
                    DnsRecordSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DnsRecordSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> DnsRecordSlot {
        DnsRecordSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            host: self.host.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a dns record slot in status planned.
pub fn new_dns_record_slot(code: &str, name: &str) -> DnsRecordSlot {
    DnsRecordSlot {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        host: "".to_string(),
        quantity: 0,
        status: DnsRecordSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dns_record_slot("code-452", "name-544");
        assert_eq!(item.status, DnsRecordSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_dns_record_slot("code-452", "name-544");
        if item.move_to(DnsRecordSlotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordSlotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dns_record_slot("code-452", "name-544");
        item.status = DnsRecordSlotStatus::Cancelled;
        assert!(
            item.move_to(DnsRecordSlotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
