use super::*;

use acme_kit::text;

/// Where a forecast proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ForecastProposalStatus {
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

impl std::fmt::Display for ForecastProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast proposal as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastProposal {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub audience: String,
    pub currency: String,
    pub channel: String,
    pub region: String,
    pub status: ForecastProposalStatus,
}

impl ForecastProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastProposalStatus) -> bool {
        match self.status {
            ForecastProposalStatus::New => {
                match next {
                    ForecastProposalStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastProposalStatus::Queued => {
                return next == ForecastProposalStatus::Running || next == ForecastProposalStatus::Failed;
            }
            ForecastProposalStatus::Running => {
                return next == ForecastProposalStatus::Done || next == ForecastProposalStatus::Failed;
            }
            ForecastProposalStatus::Done => {
                return next == ForecastProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> ForecastProposal {
        ForecastProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            audience: self.audience.clone(),
            currency: self.currency.clone(),
            channel: self.channel.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a forecast proposal in status new.
pub fn new_forecast_proposal(code: &str, name: &str) -> ForecastProposal {
    ForecastProposal {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        audience: "".to_string(),
        currency: "".to_string(),
        channel: "".to_string(),
        region: "".to_string(),
        status: ForecastProposalStatus::New,
    }
}
