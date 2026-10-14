use super::*;

use acme_kit::text;

/// Where a measurement agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl MeasurementAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for MeasurementAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAgreement {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub budget: f64,
    pub visible: bool,
    pub ceiling: f64,
    pub segment: String,
    pub updated_at: i64,
    pub principal: String,
    pub summary: String,
    pub status: MeasurementAgreementStatus,
}

impl MeasurementAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementAgreementStatus) -> bool {
        match self.status {
            MeasurementAgreementStatus::Requested => {
                return next == MeasurementAgreementStatus::Granted || next == MeasurementAgreementStatus::Revoked;
            }
            MeasurementAgreementStatus::Granted => {
                return next == MeasurementAgreementStatus::Expired;
            }
            MeasurementAgreementStatus::Revoked => {
                match next {
                    MeasurementAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: MeasurementAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.budget - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementAgreement {
        MeasurementAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            budget: self.budget,
            visible: self.visible,
            ceiling: self.ceiling,
            segment: self.segment.clone(),
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a measurement agreement in status requested.
pub fn new_measurement_agreement(code: &str, title: &str) -> MeasurementAgreement {
    MeasurementAgreement {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        budget: 0.0,
        visible: false,
        ceiling: 0.0,
        segment: "".to_string(),
        updated_at: 0,
        principal: "".to_string(),
        summary: "".to_string(),
        status: MeasurementAgreementStatus::Requested,
    }
}
