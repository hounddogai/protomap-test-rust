use super::*;

use acme_kit::text;

/// Where a strict health check score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictHealthCheckScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictHealthCheckScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictHealthCheckScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict health check score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckScore {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub channel: String,
    pub port: i64,
    pub fingerprint: String,
    pub weight: f64,
    pub discount: f64,
    pub owner_id: String,
    pub maximum: f64,
    pub tenant_id: String,
    pub status: StrictHealthCheckScoreStatus,
}

impl StrictHealthCheckScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictHealthCheckScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictHealthCheckScoreStatus) -> bool {
        match self.status {
            StrictHealthCheckScoreStatus::Proposed => {
                return next == StrictHealthCheckScoreStatus::Approved
                    || next == StrictHealthCheckScoreStatus::Rejected;
            }
            StrictHealthCheckScoreStatus::Approved => {
                match next {
                    StrictHealthCheckScoreStatus::Rejected | StrictHealthCheckScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckScoreStatus::Rejected => {
                return next == StrictHealthCheckScoreStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictHealthCheckScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict health check score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        (self.maximum * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictHealthCheckScore {
        StrictHealthCheckScore {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            channel: self.channel.clone(),
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            weight: self.weight,
            discount: self.discount,
            owner_id: self.owner_id.clone(),
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a strict health check score in status proposed.
pub fn new_strict_health_check_score(code: &str, label: &str) -> StrictHealthCheckScore {
    StrictHealthCheckScore {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        channel: "".to_string(),
        port: 0,
        fingerprint: "".to_string(),
        weight: 0.0,
        discount: 0.0,
        owner_id: "".to_string(),
        maximum: 0.0,
        tenant_id: "".to_string(),
        status: StrictHealthCheckScoreStatus::Proposed,
    }
}
