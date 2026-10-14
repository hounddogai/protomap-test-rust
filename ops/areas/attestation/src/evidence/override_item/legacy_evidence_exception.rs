use super::*;

use acme_kit::text;

/// Where a legacy evidence exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEvidenceExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyEvidenceExceptionStatus {
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

impl std::fmt::Display for LegacyEvidenceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy evidence exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceException {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub region: String,
    pub minimum: f64,
    pub pinned: bool,
    pub tenant_id: String,
    pub margin: f64,
    pub status: LegacyEvidenceExceptionStatus,
}

impl LegacyEvidenceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEvidenceExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEvidenceExceptionStatus) -> bool {
        match self.status {
            LegacyEvidenceExceptionStatus::New => {
                return next == LegacyEvidenceExceptionStatus::Queued;
            }
            LegacyEvidenceExceptionStatus::Queued => {
                match next {
                    LegacyEvidenceExceptionStatus::Running | LegacyEvidenceExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceExceptionStatus::Running => {
                match next {
                    LegacyEvidenceExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceExceptionStatus::Done => {
                return next == LegacyEvidenceExceptionStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEvidenceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy evidence exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy evidence exception", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.amount + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEvidenceException {
        LegacyEvidenceException {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            region: self.region.clone(),
            minimum: self.minimum,
            pinned: self.pinned,
            tenant_id: self.tenant_id.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy evidence exception in status new.
pub fn new_legacy_evidence_exception(code: &str, name: &str) -> LegacyEvidenceException {
    LegacyEvidenceException {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        region: "".to_string(),
        minimum: 0.0,
        pinned: false,
        tenant_id: "".to_string(),
        margin: 0.0,
        status: LegacyEvidenceExceptionStatus::New,
    }
}
