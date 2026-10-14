use super::*;

use acme_kit::text;

/// Where a legacy runbook agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRunbookAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyRunbookAgreementStatus {
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

impl std::fmt::Display for LegacyRunbookAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy runbook agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAgreement {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub burst: i64,
    pub ends_at: i64,
    pub checksum: String,
    pub weight: f64,
    pub batch_size: i64,
    pub namespace: String,
    pub label: String,
    pub status: LegacyRunbookAgreementStatus,
}

impl LegacyRunbookAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRunbookAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRunbookAgreementStatus) -> bool {
        match self.status {
            LegacyRunbookAgreementStatus::New => {
                match next {
                    LegacyRunbookAgreementStatus::Running | LegacyRunbookAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRunbookAgreementStatus::Queued => {
                return next == LegacyRunbookAgreementStatus::Failed;
            }
            LegacyRunbookAgreementStatus::Running => {
                match next {
                    LegacyRunbookAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRunbookAgreementStatus::Done => {
                match next {
                    LegacyRunbookAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyRunbookAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy runbook agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.batch_size as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRunbookAgreement {
        LegacyRunbookAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            burst: self.burst,
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            weight: self.weight,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy runbook agreement in status new.
pub fn new_legacy_runbook_agreement(code: &str, name: &str) -> LegacyRunbookAgreement {
    LegacyRunbookAgreement {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        burst: 0,
        ends_at: 0,
        checksum: "".to_string(),
        weight: 0.0,
        batch_size: 0,
        namespace: "".to_string(),
        label: "".to_string(),
        status: LegacyRunbookAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_runbook_agreement("code-459", "name-798");
        assert_eq!(item.status, LegacyRunbookAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_runbook_agreement("code-459", "name-798");
        if item.move_to(LegacyRunbookAgreementStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRunbookAgreementStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_runbook_agreement("code-459", "name-798");
        item.status = LegacyRunbookAgreementStatus::Failed;
        assert!(
            item.move_to(LegacyRunbookAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
