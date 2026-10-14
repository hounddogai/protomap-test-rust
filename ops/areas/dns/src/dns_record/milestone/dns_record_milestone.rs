use super::*;

use acme_kit::text;

/// Where a dns record milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordMilestoneStatus {
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

impl std::fmt::Display for DnsRecordMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMilestone {
    pub code: String,
    pub label: String,
    pub name: String,
    pub notes: String,
    pub revision: i64,
    pub status: DnsRecordMilestoneStatus,
}

impl DnsRecordMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordMilestoneStatus) -> bool {
        match self.status {
            DnsRecordMilestoneStatus::New => {
                match next {
                    DnsRecordMilestoneStatus::Queued | DnsRecordMilestoneStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordMilestoneStatus::Queued => {
                return next == DnsRecordMilestoneStatus::Running || next == DnsRecordMilestoneStatus::Done;
            }
            DnsRecordMilestoneStatus::Running => {
                return next == DnsRecordMilestoneStatus::Failed;
            }
            DnsRecordMilestoneStatus::Done => {
                return next == DnsRecordMilestoneStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> DnsRecordMilestone {
        DnsRecordMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record milestone in status new.
pub fn new_dns_record_milestone(code: &str, label: &str) -> DnsRecordMilestone {
    DnsRecordMilestone {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        notes: "".to_string(),
        revision: 0,
        status: DnsRecordMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dns_record_milestone("code-687", "label-942");
        assert_eq!(item.status, DnsRecordMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_dns_record_milestone("code-687", "label-942");
        if item.move_to(DnsRecordMilestoneStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordMilestoneStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dns_record_milestone("code-687", "label-942");
        item.status = DnsRecordMilestoneStatus::Failed;
        assert!(
            item.move_to(DnsRecordMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
