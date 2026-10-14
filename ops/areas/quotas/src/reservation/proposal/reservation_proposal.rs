use super::*;

use acme_kit::text;

/// Where a reservation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReservationProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReservationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reservation proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProposal {
    pub code: String,
    pub name: String,
    pub region: String,
    pub title: String,
    pub sequence: i64,
    pub approved: bool,
    pub description: String,
    pub expires_at: i64,
    pub reviewer: String,
    pub starts_at: i64,
    pub ceiling: f64,
    pub status: ReservationProposalStatus,
}

impl ReservationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationProposalStatus) -> bool {
        match self.status {
            ReservationProposalStatus::Requested => {
                return next == ReservationProposalStatus::Granted || next == ReservationProposalStatus::Revoked;
            }
            ReservationProposalStatus::Granted => {
                return next == ReservationProposalStatus::Revoked;
            }
            ReservationProposalStatus::Revoked => {
                match next {
                    ReservationProposalStatus::Expired => {
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

    pub fn move_to(&mut self, next: ReservationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation proposal", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.starts_at as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationProposal {
        ReservationProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            approved: self.approved,
            description: self.description.clone(),
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a reservation proposal in status requested.
pub fn new_reservation_proposal(code: &str, name: &str) -> ReservationProposal {
    ReservationProposal {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        title: "".to_string(),
        sequence: 0,
        approved: false,
        description: "".to_string(),
        expires_at: 0,
        reviewer: "".to_string(),
        starts_at: 0,
        ceiling: 0.0,
        status: ReservationProposalStatus::Requested,
    }
}
