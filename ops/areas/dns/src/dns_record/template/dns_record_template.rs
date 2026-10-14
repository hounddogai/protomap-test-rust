use super::*;

use acme_kit::text;

/// Where a dns record template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DnsRecordTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DnsRecordTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record template as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTemplate {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub host: String,
    pub contact_email: String,
    pub visible: bool,
    pub priority: i64,
    pub weight: f64,
    pub phone: String,
    pub region: String,
    pub status: DnsRecordTemplateStatus,
}

impl DnsRecordTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordTemplateStatus) -> bool {
        match self.status {
            DnsRecordTemplateStatus::Proposed => {
                return next == DnsRecordTemplateStatus::Approved || next == DnsRecordTemplateStatus::Withdrawn;
            }
            DnsRecordTemplateStatus::Approved => {
                match next {
                    DnsRecordTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordTemplateStatus::Rejected => {
                return next == DnsRecordTemplateStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.weight * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordTemplate {
        DnsRecordTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            host: self.host.clone(),
            contact_email: self.contact_email.clone(),
            visible: self.visible,
            priority: self.priority,
            weight: self.weight,
            phone: self.phone.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a dns record template in status proposed.
pub fn new_dns_record_template(code: &str, title: &str) -> DnsRecordTemplate {
    DnsRecordTemplate {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        host: "".to_string(),
        contact_email: "".to_string(),
        visible: false,
        priority: 0,
        weight: 0.0,
        phone: "".to_string(),
        region: "".to_string(),
        status: DnsRecordTemplateStatus::Proposed,
    }
}
