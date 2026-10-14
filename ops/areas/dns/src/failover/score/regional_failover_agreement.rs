use super::*;

use acme_kit::text;

/// Where a regional failover agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFailoverAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalFailoverAgreementStatus {
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

impl std::fmt::Display for RegionalFailoverAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional failover agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverAgreement {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub cost: f64,
    pub active: bool,
    pub ends_at: i64,
    pub locale: String,
    pub status: RegionalFailoverAgreementStatus,
}

impl RegionalFailoverAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFailoverAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFailoverAgreementStatus) -> bool {
        match self.status {
            RegionalFailoverAgreementStatus::New => {
                match next {
                    RegionalFailoverAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverAgreementStatus::Queued => {
                match next {
                    RegionalFailoverAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFailoverAgreementStatus::Running => {
                return next == RegionalFailoverAgreementStatus::Failed;
            }
            RegionalFailoverAgreementStatus::Done => {
                return next == RegionalFailoverAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalFailoverAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional failover agreement {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.rate + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFailoverAgreement {
        RegionalFailoverAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            cost: self.cost,
            active: self.active,
            ends_at: self.ends_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional failover agreement in status new.
pub fn new_regional_failover_agreement(code: &str, title: &str) -> RegionalFailoverAgreement {
    RegionalFailoverAgreement {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        cost: 0.0,
        active: false,
        ends_at: 0,
        locale: "".to_string(),
        status: RegionalFailoverAgreementStatus::New,
    }
}
