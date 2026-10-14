use super::*;

use acme_kit::text;

/// Where a strict sink agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSinkAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictSinkAgreementStatus {
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

impl std::fmt::Display for StrictSinkAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict sink agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreement {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub tenant_id: String,
    pub active: bool,
    pub quota_units: i64,
    pub status: StrictSinkAgreementStatus,
}

impl StrictSinkAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSinkAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSinkAgreementStatus) -> bool {
        match self.status {
            StrictSinkAgreementStatus::New => {
                return next == StrictSinkAgreementStatus::Running;
            }
            StrictSinkAgreementStatus::Queued => {
                match next {
                    StrictSinkAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSinkAgreementStatus::Running => {
                match next {
                    StrictSinkAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSinkAgreementStatus::Done => {
                match next {
                    StrictSinkAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictSinkAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict sink agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictSinkAgreement {
        StrictSinkAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            tenant_id: self.tenant_id.clone(),
            active: self.active,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict sink agreement in status new.
pub fn new_strict_sink_agreement(code: &str, title: &str) -> StrictSinkAgreement {
    StrictSinkAgreement {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        tenant_id: "".to_string(),
        active: false,
        quota_units: 0,
        status: StrictSinkAgreementStatus::New,
    }
}
