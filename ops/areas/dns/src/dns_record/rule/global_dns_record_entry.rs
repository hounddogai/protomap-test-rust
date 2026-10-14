use super::*;

use acme_kit::text;

/// Where a global dns record entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDnsRecordEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalDnsRecordEntryStatus {
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

impl std::fmt::Display for GlobalDnsRecordEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global dns record entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordEntry {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub rotation_days: i64,
    pub port: i64,
    pub budget: f64,
    pub country: String,
    pub contact_email: String,
    pub status: GlobalDnsRecordEntryStatus,
}

impl GlobalDnsRecordEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDnsRecordEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDnsRecordEntryStatus) -> bool {
        match self.status {
            GlobalDnsRecordEntryStatus::Planned => {
                match next {
                    GlobalDnsRecordEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDnsRecordEntryStatus::Scheduled => {
                return next == GlobalDnsRecordEntryStatus::InProgress;
            }
            GlobalDnsRecordEntryStatus::InProgress => {
                match next {
                    GlobalDnsRecordEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDnsRecordEntryStatus::Completed => {
                match next {
                    GlobalDnsRecordEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalDnsRecordEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global dns record entry {} cannot move from {} to {}",
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
        format!("{} {}", "global dns record entry", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.budget * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDnsRecordEntry {
        GlobalDnsRecordEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            port: self.port,
            budget: self.budget,
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global dns record entry in status planned.
pub fn new_global_dns_record_entry(code: &str, title: &str) -> GlobalDnsRecordEntry {
    GlobalDnsRecordEntry {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        rotation_days: 0,
        port: 0,
        budget: 0.0,
        country: "".to_string(),
        contact_email: "".to_string(),
        status: GlobalDnsRecordEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_dns_record_entry("code-971", "title-17");
        assert_eq!(item.status, GlobalDnsRecordEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_global_dns_record_entry("code-971", "title-17");
        if item.move_to(GlobalDnsRecordEntryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalDnsRecordEntryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_dns_record_entry("code-971", "title-17");
        item.status = GlobalDnsRecordEntryStatus::Cancelled;
        assert!(
            item.move_to(GlobalDnsRecordEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
