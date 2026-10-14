use super::*;

use acme_kit::text;

/// Where a capped query agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQueryAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedQueryAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CappedQueryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped query agreement as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAgreement {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub external_ref: String,
    pub category: String,
    pub status: CappedQueryAgreementStatus,
}

impl CappedQueryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQueryAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQueryAgreementStatus) -> bool {
        match self.status {
            CappedQueryAgreementStatus::Idle => {
                match next {
                    CappedQueryAgreementStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryAgreementStatus::Warming => {
                return next == CappedQueryAgreementStatus::Stopped;
            }
            CappedQueryAgreementStatus::Ready => {
                return next == CappedQueryAgreementStatus::Draining || next == CappedQueryAgreementStatus::Stopped;
            }
            CappedQueryAgreementStatus::Draining => {
                match next {
                    CappedQueryAgreementStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedQueryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped query agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> CappedQueryAgreement {
        CappedQueryAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            external_ref: self.external_ref.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped query agreement in status idle.
pub fn new_capped_query_agreement(code: &str, name: &str) -> CappedQueryAgreement {
    CappedQueryAgreement {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        external_ref: "".to_string(),
        category: "".to_string(),
        status: CappedQueryAgreementStatus::Idle,
    }
}
