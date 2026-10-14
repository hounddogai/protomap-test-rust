use super::*;

use acme_kit::text;

/// Where a probe agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ProbeAgreementStatus {
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

impl std::fmt::Display for ProbeAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAgreement {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub ratio: f64,
    pub grace_days: i64,
    pub status: ProbeAgreementStatus,
}

impl ProbeAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeAgreementStatus) -> bool {
        match self.status {
            ProbeAgreementStatus::Planned => {
                return next == ProbeAgreementStatus::Completed;
            }
            ProbeAgreementStatus::Scheduled => {
                return next == ProbeAgreementStatus::InProgress;
            }
            ProbeAgreementStatus::InProgress => {
                return next == ProbeAgreementStatus::Completed || next == ProbeAgreementStatus::Cancelled;
            }
            ProbeAgreementStatus::Completed => {
                match next {
                    ProbeAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ProbeAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe agreement", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.grace_days as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeAgreement {
        ProbeAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            ratio: self.ratio,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a probe agreement in status planned.
pub fn new_probe_agreement(code: &str, name: &str) -> ProbeAgreement {
    ProbeAgreement {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        ratio: 0.0,
        grace_days: 0,
        status: ProbeAgreementStatus::Planned,
    }
}
