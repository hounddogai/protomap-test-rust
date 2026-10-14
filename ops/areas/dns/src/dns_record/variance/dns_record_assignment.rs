use super::*;

use acme_kit::text;

/// Where a dns record assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for DnsRecordAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record assignment as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAssignment {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub approved: bool,
    pub ends_at: i64,
    pub host: String,
    pub status: DnsRecordAssignmentStatus,
}

impl DnsRecordAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordAssignmentStatus) -> bool {
        match self.status {
            DnsRecordAssignmentStatus::New => {
                return next == DnsRecordAssignmentStatus::Queued;
            }
            DnsRecordAssignmentStatus::Queued => {
                return next == DnsRecordAssignmentStatus::Done || next == DnsRecordAssignmentStatus::Failed;
            }
            DnsRecordAssignmentStatus::Running => {
                match next {
                    DnsRecordAssignmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAssignmentStatus::Done => {
                return next == DnsRecordAssignmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record assignment {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> DnsRecordAssignment {
        DnsRecordAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            ends_at: self.ends_at,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record assignment in status new.
pub fn new_dns_record_assignment(code: &str, label: &str) -> DnsRecordAssignment {
    DnsRecordAssignment {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        approved: false,
        ends_at: 0,
        host: "".to_string(),
        status: DnsRecordAssignmentStatus::New,
    }
}
