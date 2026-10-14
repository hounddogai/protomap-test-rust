use super::*;

use acme_kit::text;

/// Where a slo agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SloAgreementStatus {
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

impl std::fmt::Display for SloAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreement {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub source: String,
    pub region: String,
    pub maximum: f64,
    pub category: String,
    pub status: SloAgreementStatus,
}

impl SloAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloAgreementStatus) -> bool {
        match self.status {
            SloAgreementStatus::New => {
                return next == SloAgreementStatus::Done;
            }
            SloAgreementStatus::Queued => {
                match next {
                    SloAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloAgreementStatus::Running => {
                return next == SloAgreementStatus::Done || next == SloAgreementStatus::Failed;
            }
            SloAgreementStatus::Done => {
                match next {
                    SloAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: SloAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> SloAgreement {
        SloAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            maximum: self.maximum,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a slo agreement in status new.
pub fn new_slo_agreement(code: &str, label: &str) -> SloAgreement {
    SloAgreement {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        source: "".to_string(),
        region: "".to_string(),
        maximum: 0.0,
        category: "".to_string(),
        status: SloAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_slo_agreement("code-50", "label-868");
        assert_eq!(item.status, SloAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_slo_agreement("code-50", "label-868");
        if item.move_to(SloAgreementStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloAgreementStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_slo_agreement("code-50", "label-868");
        item.status = SloAgreementStatus::Failed;
        assert!(item.move_to(SloAgreementStatus::New).is_err(), "expected move to fail");
    }
}
