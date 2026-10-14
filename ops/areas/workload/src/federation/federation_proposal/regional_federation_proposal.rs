use super::*;

use acme_kit::text;

/// Where a regional federation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFederationProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalFederationProposalStatus {
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

impl std::fmt::Display for RegionalFederationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional federation proposal as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposal {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub pinned: bool,
    pub window_days: i64,
    pub checksum: String,
    pub status: RegionalFederationProposalStatus,
}

impl RegionalFederationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFederationProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFederationProposalStatus) -> bool {
        match self.status {
            RegionalFederationProposalStatus::Draft => {
                match next {
                    RegionalFederationProposalStatus::Active | RegionalFederationProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFederationProposalStatus::Pending => {
                match next {
                    RegionalFederationProposalStatus::Active | RegionalFederationProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFederationProposalStatus::Active => {
                match next {
                    RegionalFederationProposalStatus::Suspended | RegionalFederationProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFederationProposalStatus::Suspended => {
                return next == RegionalFederationProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalFederationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional federation proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional federation proposal", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.window_days as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFederationProposal {
        RegionalFederationProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            pinned: self.pinned,
            window_days: self.window_days,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional federation proposal in status draft.
pub fn new_regional_federation_proposal(code: &str, title: &str) -> RegionalFederationProposal {
    RegionalFederationProposal {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        pinned: false,
        window_days: 0,
        checksum: "".to_string(),
        status: RegionalFederationProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_federation_proposal("code-37", "title-432");
        assert_eq!(item.status, RegionalFederationProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_regional_federation_proposal("code-37", "title-432");
        if item.move_to(RegionalFederationProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFederationProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_federation_proposal("code-37", "title-432");
        item.status = RegionalFederationProposalStatus::Closed;
        assert!(
            item.move_to(RegionalFederationProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
