use super::*;

use acme_kit::text;

/// Where a sink score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SinkScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SinkScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkScore {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub locale: String,
    pub external_ref: String,
    pub status: SinkScoreStatus,
}

impl SinkScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkScoreStatus) -> bool {
        match self.status {
            SinkScoreStatus::Created => {
                match next {
                    SinkScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkScoreStatus::Validated => {
                match next {
                    SinkScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkScoreStatus::Published => {
                return next == SinkScoreStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink score", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> SinkScore {
        SinkScore {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a sink score in status created.
pub fn new_sink_score(code: &str, title: &str) -> SinkScore {
    SinkScore {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        locale: "".to_string(),
        external_ref: "".to_string(),
        status: SinkScoreStatus::Created,
    }
}
