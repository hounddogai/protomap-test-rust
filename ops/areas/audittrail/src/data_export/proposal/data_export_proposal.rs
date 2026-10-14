use super::*;

use acme_kit::text;

/// Where a data export proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DataExportProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DataExportProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposal {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub issuer: String,
    pub ttl_seconds: i64,
    pub ratio: f64,
    pub retries: i64,
    pub status: DataExportProposalStatus,
}

impl DataExportProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportProposalStatus) -> bool {
        match self.status {
            DataExportProposalStatus::Idle => {
                return next == DataExportProposalStatus::Warming;
            }
            DataExportProposalStatus::Warming => {
                return next == DataExportProposalStatus::Draining;
            }
            DataExportProposalStatus::Ready => {
                return next == DataExportProposalStatus::Draining;
            }
            DataExportProposalStatus::Draining => {
                return next == DataExportProposalStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export proposal", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ratio + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportProposal {
        DataExportProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            issuer: self.issuer.clone(),
            ttl_seconds: self.ttl_seconds,
            ratio: self.ratio,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a data export proposal in status idle.
pub fn new_data_export_proposal(code: &str, name: &str) -> DataExportProposal {
    DataExportProposal {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        issuer: "".to_string(),
        ttl_seconds: 0,
        ratio: 0.0,
        retries: 0,
        status: DataExportProposalStatus::Idle,
    }
}
