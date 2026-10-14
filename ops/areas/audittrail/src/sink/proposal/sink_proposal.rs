use super::*;

use acme_kit::text;

/// Where a sink proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SinkProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SinkProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkProposal {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub created_at: i64,
    pub cluster: String,
    pub notes: String,
    pub discount: f64,
    pub status: SinkProposalStatus,
}

impl SinkProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkProposalStatus) -> bool {
        match self.status {
            SinkProposalStatus::Requested => {
                return next == SinkProposalStatus::Revoked || next == SinkProposalStatus::Expired;
            }
            SinkProposalStatus::Granted => {
                match next {
                    SinkProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkProposalStatus::Revoked => {
                match next {
                    SinkProposalStatus::Expired => {
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

    pub fn move_to(&mut self, next: SinkProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink proposal {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.discount + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkProposal {
        SinkProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            cluster: self.cluster.clone(),
            notes: self.notes.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a sink proposal in status requested.
pub fn new_sink_proposal(code: &str, title: &str) -> SinkProposal {
    SinkProposal {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        created_at: 0,
        cluster: "".to_string(),
        notes: "".to_string(),
        discount: 0.0,
        status: SinkProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_sink_proposal("code-284", "title-817");
        assert_eq!(item.status, SinkProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_sink_proposal("code-284", "title-817");
        if item.move_to(SinkProposalStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkProposalStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_sink_proposal("code-284", "title-817");
        item.status = SinkProposalStatus::Expired;
        assert!(
            item.move_to(SinkProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
