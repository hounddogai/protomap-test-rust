use super::*;

use acme_kit::text;

/// Where a quote proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl QuoteProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for QuoteProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProposal {
    pub code: String,
    pub name: String,
    pub title: String,
    pub budget: f64,
    pub tags: Vec<String>,
    pub cost: f64,
    pub status: QuoteProposalStatus,
}

impl QuoteProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteProposalStatus) -> bool {
        match self.status {
            QuoteProposalStatus::Requested => {
                return next == QuoteProposalStatus::Granted || next == QuoteProposalStatus::Expired;
            }
            QuoteProposalStatus::Granted => {
                match next {
                    QuoteProposalStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteProposalStatus::Revoked => {
                match next {
                    QuoteProposalStatus::Expired => {
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

    pub fn move_to(&mut self, next: QuoteProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote proposal", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.cost * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteProposal {
        QuoteProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            budget: self.budget,
            tags: self.tags.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a quote proposal in status requested.
pub fn new_quote_proposal(code: &str, name: &str) -> QuoteProposal {
    QuoteProposal {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        budget: 0.0,
        tags: Vec::new(),
        cost: 0.0,
        status: QuoteProposalStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_quote_proposal("code-787", "name-389");
        assert_eq!(item.status, QuoteProposalStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_quote_proposal("code-787", "name-389");
        if item.move_to(QuoteProposalStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteProposalStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_quote_proposal("code-787", "name-389");
        item.status = QuoteProposalStatus::Expired;
        assert!(
            item.move_to(QuoteProposalStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
