use super::*;

use acme_kit::text;

/// Where a rollback agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollbackAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollbackAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback agreement as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAgreement {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub external_ref: String,
    pub segment: String,
    pub status: RollbackAgreementStatus,
}

impl RollbackAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackAgreementStatus) -> bool {
        match self.status {
            RollbackAgreementStatus::Created => {
                match next {
                    RollbackAgreementStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAgreementStatus::Validated => {
                match next {
                    RollbackAgreementStatus::Published | RollbackAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackAgreementStatus::Published => {
                return next == RollbackAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> RollbackAgreement {
        RollbackAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            external_ref: self.external_ref.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollback agreement in status created.
pub fn new_rollback_agreement(code: &str, label: &str) -> RollbackAgreement {
    RollbackAgreement {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        external_ref: "".to_string(),
        segment: "".to_string(),
        status: RollbackAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollback_agreement("code-583", "label-671");
        assert_eq!(item.status, RollbackAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_rollback_agreement("code-583", "label-671");
        if item.move_to(RollbackAgreementStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackAgreementStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollback_agreement("code-583", "label-671");
        item.status = RollbackAgreementStatus::Retired;
        assert!(
            item.move_to(RollbackAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
