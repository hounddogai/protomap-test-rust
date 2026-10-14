use super::*;

use acme_kit::text;

/// Where a suspension agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SuspensionAgreementStatus {
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

impl std::fmt::Display for SuspensionAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension agreement as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreement {
    pub code: String,
    pub name: String,
    pub host: String,
    pub phone: String,
    pub capacity: i64,
    pub label: String,
    pub weight: f64,
    pub discount: f64,
    pub status: SuspensionAgreementStatus,
}

impl SuspensionAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionAgreementStatus) -> bool {
        match self.status {
            SuspensionAgreementStatus::Planned => {
                match next {
                    SuspensionAgreementStatus::Completed | SuspensionAgreementStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAgreementStatus::Scheduled => {
                return next == SuspensionAgreementStatus::Completed;
            }
            SuspensionAgreementStatus::InProgress => {
                return next == SuspensionAgreementStatus::Completed;
            }
            SuspensionAgreementStatus::Completed => {
                match next {
                    SuspensionAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SuspensionAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension agreement", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionAgreement {
        SuspensionAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            phone: self.phone.clone(),
            capacity: self.capacity,
            label: self.label.clone(),
            weight: self.weight,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension agreement in status planned.
pub fn new_suspension_agreement(code: &str, name: &str) -> SuspensionAgreement {
    SuspensionAgreement {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        phone: "".to_string(),
        capacity: 0,
        label: "".to_string(),
        weight: 0.0,
        discount: 0.0,
        status: SuspensionAgreementStatus::Planned,
    }
}
