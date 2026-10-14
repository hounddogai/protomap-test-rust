use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual identity proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIdentityProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualIdentityProposalStatus {
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

impl std::fmt::Display for ManualIdentityProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual identity proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposal {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub grace_days: i64,
    pub principal: String,
    pub attributes: BTreeMap<String, String>,
    pub checksum: String,
    pub approved: bool,
    pub scope: String,
    pub version: i64,
    pub status: ManualIdentityProposalStatus,
}

impl ManualIdentityProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIdentityProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIdentityProposalStatus) -> bool {
        match self.status {
            ManualIdentityProposalStatus::Draft => {
                return next == ManualIdentityProposalStatus::Active;
            }
            ManualIdentityProposalStatus::Pending => {
                return next == ManualIdentityProposalStatus::Active;
            }
            ManualIdentityProposalStatus::Active => {
                return next == ManualIdentityProposalStatus::Suspended;
            }
            ManualIdentityProposalStatus::Suspended => {
                return next == ManualIdentityProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualIdentityProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual identity proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.version as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIdentityProposal {
        ManualIdentityProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            attributes: self.attributes.clone(),
            checksum: self.checksum.clone(),
            approved: self.approved,
            scope: self.scope.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual identity proposal in status draft.
pub fn new_manual_identity_proposal(code: &str, label: &str) -> ManualIdentityProposal {
    ManualIdentityProposal {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        grace_days: 0,
        principal: "".to_string(),
        attributes: BTreeMap::new(),
        checksum: "".to_string(),
        approved: false,
        scope: "".to_string(),
        version: 0,
        status: ManualIdentityProposalStatus::Draft,
    }
}
