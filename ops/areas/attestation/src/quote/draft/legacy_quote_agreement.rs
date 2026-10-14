use super::*;

use acme_kit::text;

/// Where a legacy quote agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQuoteAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyQuoteAgreementStatus {
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

impl std::fmt::Display for LegacyQuoteAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy quote agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteAgreement {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub budget: f64,
    pub summary: String,
    pub status: LegacyQuoteAgreementStatus,
}

impl LegacyQuoteAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQuoteAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQuoteAgreementStatus) -> bool {
        match self.status {
            LegacyQuoteAgreementStatus::New => {
                match next {
                    LegacyQuoteAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQuoteAgreementStatus::Queued => {
                match next {
                    LegacyQuoteAgreementStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQuoteAgreementStatus::Running => {
                return next == LegacyQuoteAgreementStatus::Done || next == LegacyQuoteAgreementStatus::Failed;
            }
            LegacyQuoteAgreementStatus::Done => {
                match next {
                    LegacyQuoteAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyQuoteAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy quote agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> LegacyQuoteAgreement {
        LegacyQuoteAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            budget: self.budget,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy quote agreement in status new.
pub fn new_legacy_quote_agreement(code: &str, name: &str) -> LegacyQuoteAgreement {
    LegacyQuoteAgreement {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        budget: 0.0,
        summary: "".to_string(),
        status: LegacyQuoteAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_quote_agreement("code-549", "name-262");
        assert_eq!(item.status, LegacyQuoteAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_legacy_quote_agreement("code-549", "name-262");
        if item.move_to(LegacyQuoteAgreementStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyQuoteAgreementStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_quote_agreement("code-549", "name-262");
        item.status = LegacyQuoteAgreementStatus::Failed;
        assert!(
            item.move_to(LegacyQuoteAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
