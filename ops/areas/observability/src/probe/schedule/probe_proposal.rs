use super::*;

use acme_kit::text;

/// Where a probe proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ProbeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeProposal {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub key_id: String,
    pub timezone: String,
    pub country: String,
    pub maximum: f64,
    pub burst: i64,
    pub status: ProbeProposalStatus,
}

impl ProbeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeProposalStatus) -> bool {
        match self.status {
            ProbeProposalStatus::Draft => {
                return next == ProbeProposalStatus::Active || next == ProbeProposalStatus::Closed;
            }
            ProbeProposalStatus::Pending => {
                return next == ProbeProposalStatus::Closed;
            }
            ProbeProposalStatus::Active => {
                return next == ProbeProposalStatus::Closed;
            }
            ProbeProposalStatus::Suspended => {
                match next {
                    ProbeProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: ProbeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe proposal", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.discount - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeProposal {
        ProbeProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            key_id: self.key_id.clone(),
            timezone: self.timezone.clone(),
            country: self.country.clone(),
            maximum: self.maximum,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a probe proposal in status draft.
pub fn new_probe_proposal(code: &str, name: &str) -> ProbeProposal {
    ProbeProposal {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        key_id: "".to_string(),
        timezone: "".to_string(),
        country: "".to_string(),
        maximum: 0.0,
        burst: 0,
        status: ProbeProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_proposal("code-349", "name-601");
        assert_eq!(item.status, ProbeProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_probe_proposal("code-349", "name-601");
        if item.move_to(ProbeProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_proposal("code-349", "name-601");
        item.status = ProbeProposalStatus::Closed;
        assert!(
            item.move_to(ProbeProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
