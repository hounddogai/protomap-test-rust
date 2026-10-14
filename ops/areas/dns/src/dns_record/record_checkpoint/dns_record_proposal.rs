use super::*;

use acme_kit::text;

/// Where a dns record proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DnsRecordProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DnsRecordProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProposal {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub ends_at: i64,
    pub visible: bool,
    pub port: i64,
    pub burst: i64,
    pub window_days: i64,
    pub quantity: i64,
    pub status: DnsRecordProposalStatus,
}

impl DnsRecordProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordProposalStatus) -> bool {
        match self.status {
            DnsRecordProposalStatus::Open => {
                return next == DnsRecordProposalStatus::InReview;
            }
            DnsRecordProposalStatus::InReview => {
                return next == DnsRecordProposalStatus::Resolved;
            }
            DnsRecordProposalStatus::Resolved => {
                return next == DnsRecordProposalStatus::Reopened;
            }
            DnsRecordProposalStatus::Reopened => {
                match next {
                    DnsRecordProposalStatus::Archived => {
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

    pub fn move_to(&mut self, next: DnsRecordProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record proposal {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordProposal {
        DnsRecordProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            ends_at: self.ends_at,
            visible: self.visible,
            port: self.port,
            burst: self.burst,
            window_days: self.window_days,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record proposal in status open.
pub fn new_dns_record_proposal(code: &str, name: &str) -> DnsRecordProposal {
    DnsRecordProposal {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        ends_at: 0,
        visible: false,
        port: 0,
        burst: 0,
        window_days: 0,
        quantity: 0,
        status: DnsRecordProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dns_record_proposal("code-847", "name-19");
        assert_eq!(item.status, DnsRecordProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_dns_record_proposal("code-847", "name-19");
        if item.move_to(DnsRecordProposalStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordProposalStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dns_record_proposal("code-847", "name-19");
        item.status = DnsRecordProposalStatus::Archived;
        assert!(
            item.move_to(DnsRecordProposalStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
