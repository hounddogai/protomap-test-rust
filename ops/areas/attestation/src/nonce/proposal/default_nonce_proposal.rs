use super::*;

use acme_kit::text;

/// Where a default nonce proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultNonceProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultNonceProposalStatus {
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

impl std::fmt::Display for DefaultNonceProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default nonce proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceProposal {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub rotation_days: i64,
    pub amount: f64,
    pub maximum: f64,
    pub limit: i64,
    pub discount: f64,
    pub archived: bool,
    pub status: DefaultNonceProposalStatus,
}

impl DefaultNonceProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultNonceProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultNonceProposalStatus) -> bool {
        match self.status {
            DefaultNonceProposalStatus::Draft => {
                match next {
                    DefaultNonceProposalStatus::Active | DefaultNonceProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultNonceProposalStatus::Pending => {
                match next {
                    DefaultNonceProposalStatus::Suspended | DefaultNonceProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultNonceProposalStatus::Active => {
                return next == DefaultNonceProposalStatus::Closed;
            }
            DefaultNonceProposalStatus::Suspended => {
                match next {
                    DefaultNonceProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultNonceProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default nonce proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.maximum - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultNonceProposal {
        DefaultNonceProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            rotation_days: self.rotation_days,
            amount: self.amount,
            maximum: self.maximum,
            limit: self.limit,
            discount: self.discount,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default nonce proposal in status draft.
pub fn new_default_nonce_proposal(code: &str, label: &str) -> DefaultNonceProposal {
    DefaultNonceProposal {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        rotation_days: 0,
        amount: 0.0,
        maximum: 0.0,
        limit: 0,
        discount: 0.0,
        archived: false,
        status: DefaultNonceProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_nonce_proposal("code-119", "label-957");
        assert_eq!(item.status, DefaultNonceProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_default_nonce_proposal("code-119", "label-957");
        if item.move_to(DefaultNonceProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultNonceProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_nonce_proposal("code-119", "label-957");
        item.status = DefaultNonceProposalStatus::Closed;
        assert!(
            item.move_to(DefaultNonceProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
