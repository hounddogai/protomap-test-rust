use super::*;

use acme_kit::text;

/// Where a lenient measurement rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMeasurementRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientMeasurementRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientMeasurementRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient measurement rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollup {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub issuer: String,
    pub external_ref: String,
    pub cipher: String,
    pub margin: f64,
    pub visible: bool,
    pub status: LenientMeasurementRollupStatus,
}

impl LenientMeasurementRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMeasurementRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMeasurementRollupStatus) -> bool {
        match self.status {
            LenientMeasurementRollupStatus::Created => {
                return next == LenientMeasurementRollupStatus::Retired;
            }
            LenientMeasurementRollupStatus::Validated => {
                return next == LenientMeasurementRollupStatus::Published
                    || next == LenientMeasurementRollupStatus::Retired;
            }
            LenientMeasurementRollupStatus::Published => {
                match next {
                    LenientMeasurementRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientMeasurementRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient measurement rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient measurement rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> LenientMeasurementRollup {
        LenientMeasurementRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            external_ref: self.external_ref.clone(),
            cipher: self.cipher.clone(),
            margin: self.margin,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient measurement rollup in status created.
pub fn new_lenient_measurement_rollup(code: &str, label: &str) -> LenientMeasurementRollup {
    LenientMeasurementRollup {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        issuer: "".to_string(),
        external_ref: "".to_string(),
        cipher: "".to_string(),
        margin: 0.0,
        visible: false,
        status: LenientMeasurementRollupStatus::Created,
    }
}
