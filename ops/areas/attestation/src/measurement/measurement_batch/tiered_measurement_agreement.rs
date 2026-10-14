use super::*;

use acme_kit::text;

/// Where a tiered measurement agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMeasurementAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredMeasurementAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredMeasurementAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered measurement agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreement {
    pub code: String,
    pub name: String,
    pub title: String,
    pub pinned: bool,
    pub grace_days: i64,
    pub status: TieredMeasurementAgreementStatus,
}

impl TieredMeasurementAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMeasurementAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMeasurementAgreementStatus) -> bool {
        match self.status {
            TieredMeasurementAgreementStatus::Created => {
                return next == TieredMeasurementAgreementStatus::Published
                    || next == TieredMeasurementAgreementStatus::Retired;
            }
            TieredMeasurementAgreementStatus::Validated => {
                return next == TieredMeasurementAgreementStatus::Published;
            }
            TieredMeasurementAgreementStatus::Published => {
                return next == TieredMeasurementAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredMeasurementAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered measurement agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> TieredMeasurementAgreement {
        TieredMeasurementAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered measurement agreement in status created.
pub fn new_tiered_measurement_agreement(code: &str, name: &str) -> TieredMeasurementAgreement {
    TieredMeasurementAgreement {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        pinned: false,
        grace_days: 0,
        status: TieredMeasurementAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_measurement_agreement("code-918", "name-533");
        assert_eq!(item.status, TieredMeasurementAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tiered_measurement_agreement("code-918", "name-533");
        if item.move_to(TieredMeasurementAgreementStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredMeasurementAgreementStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_measurement_agreement("code-918", "name-533");
        item.status = TieredMeasurementAgreementStatus::Retired;
        assert!(
            item.move_to(TieredMeasurementAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
