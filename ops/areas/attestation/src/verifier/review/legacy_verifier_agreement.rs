use super::*;

use acme_kit::text;

/// Where a legacy verifier agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyVerifierAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyVerifierAgreementStatus {
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

impl std::fmt::Display for LegacyVerifierAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy verifier agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierAgreement {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub rotation_days: i64,
    pub external_ref: String,
    pub pinned: bool,
    pub minimum: f64,
    pub sequence: i64,
    pub margin: f64,
    pub label: String,
    pub status: LegacyVerifierAgreementStatus,
}

impl LegacyVerifierAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyVerifierAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyVerifierAgreementStatus) -> bool {
        match self.status {
            LegacyVerifierAgreementStatus::New => {
                match next {
                    LegacyVerifierAgreementStatus::Queued | LegacyVerifierAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyVerifierAgreementStatus::Queued => {
                match next {
                    LegacyVerifierAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyVerifierAgreementStatus::Running => {
                return next == LegacyVerifierAgreementStatus::Failed;
            }
            LegacyVerifierAgreementStatus::Done => {
                match next {
                    LegacyVerifierAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyVerifierAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy verifier agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.minimum - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyVerifierAgreement {
        LegacyVerifierAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            rotation_days: self.rotation_days,
            external_ref: self.external_ref.clone(),
            pinned: self.pinned,
            minimum: self.minimum,
            sequence: self.sequence,
            margin: self.margin,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy verifier agreement in status new.
pub fn new_legacy_verifier_agreement(code: &str, name: &str) -> LegacyVerifierAgreement {
    LegacyVerifierAgreement {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        rotation_days: 0,
        external_ref: "".to_string(),
        pinned: false,
        minimum: 0.0,
        sequence: 0,
        margin: 0.0,
        label: "".to_string(),
        status: LegacyVerifierAgreementStatus::New,
    }
}
