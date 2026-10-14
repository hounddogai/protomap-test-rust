use super::*;

use acme_kit::text;

/// Where a manual dns record template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDnsRecordTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualDnsRecordTemplateStatus {
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

impl std::fmt::Display for ManualDnsRecordTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual dns record template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordTemplate {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub pinned: bool,
    pub region: String,
    pub tenant_id: String,
    pub weight: f64,
    pub name: String,
    pub status: ManualDnsRecordTemplateStatus,
}

impl ManualDnsRecordTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDnsRecordTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDnsRecordTemplateStatus) -> bool {
        match self.status {
            ManualDnsRecordTemplateStatus::Planned => {
                match next {
                    ManualDnsRecordTemplateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDnsRecordTemplateStatus::Scheduled => {
                match next {
                    ManualDnsRecordTemplateStatus::InProgress | ManualDnsRecordTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDnsRecordTemplateStatus::InProgress => {
                return next == ManualDnsRecordTemplateStatus::Completed
                    || next == ManualDnsRecordTemplateStatus::Cancelled;
            }
            ManualDnsRecordTemplateStatus::Completed => {
                return next == ManualDnsRecordTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDnsRecordTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual dns record template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDnsRecordTemplate {
        ManualDnsRecordTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            pinned: self.pinned,
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            weight: self.weight,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual dns record template in status planned.
pub fn new_manual_dns_record_template(code: &str, title: &str) -> ManualDnsRecordTemplate {
    ManualDnsRecordTemplate {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        pinned: false,
        region: "".to_string(),
        tenant_id: "".to_string(),
        weight: 0.0,
        name: "".to_string(),
        status: ManualDnsRecordTemplateStatus::Planned,
    }
}
