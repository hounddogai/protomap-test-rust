use super::*;

use acme_kit::text;

/// Where a failover agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FailoverAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FailoverAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover agreement as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAgreement {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub key_id: String,
    pub rotation_days: i64,
    pub title: String,
    pub notes: String,
    pub maximum: f64,
    pub status: FailoverAgreementStatus,
}

impl FailoverAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverAgreementStatus) -> bool {
        match self.status {
            FailoverAgreementStatus::Created => {
                match next {
                    FailoverAgreementStatus::Validated | FailoverAgreementStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverAgreementStatus::Validated => {
                match next {
                    FailoverAgreementStatus::Published | FailoverAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverAgreementStatus::Published => {
                return next == FailoverAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rotation_days as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverAgreement {
        FailoverAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            key_id: self.key_id.clone(),
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            notes: self.notes.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover agreement in status created.
pub fn new_failover_agreement(code: &str, label: &str) -> FailoverAgreement {
    FailoverAgreement {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        key_id: "".to_string(),
        rotation_days: 0,
        title: "".to_string(),
        notes: "".to_string(),
        maximum: 0.0,
        status: FailoverAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_failover_agreement("code-215", "label-798");
        assert_eq!(item.status, FailoverAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_failover_agreement("code-215", "label-798");
        if item.move_to(FailoverAgreementStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverAgreementStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_failover_agreement("code-215", "label-798");
        item.status = FailoverAgreementStatus::Retired;
        assert!(
            item.move_to(FailoverAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
