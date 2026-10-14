use super::*;

use acme_kit::text;

/// Where a limit proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LimitProposalStatus {
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

impl std::fmt::Display for LimitProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitProposal {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub tags: Vec<String>,
    pub timezone: String,
    pub quantity: i64,
    pub status: LimitProposalStatus,
}

impl LimitProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitProposalStatus) -> bool {
        match self.status {
            LimitProposalStatus::Draft => {
                return next == LimitProposalStatus::Active || next == LimitProposalStatus::Suspended;
            }
            LimitProposalStatus::Pending => {
                match next {
                    LimitProposalStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitProposalStatus::Active => {
                match next {
                    LimitProposalStatus::Suspended | LimitProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitProposalStatus::Suspended => {
                match next {
                    LimitProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: LimitProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> LimitProposal {
        LimitProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            tags: self.tags.clone(),
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a limit proposal in status draft.
pub fn new_limit_proposal(code: &str, title: &str) -> LimitProposal {
    LimitProposal {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        tags: Vec::new(),
        timezone: "".to_string(),
        quantity: 0,
        status: LimitProposalStatus::Draft,
    }
}
