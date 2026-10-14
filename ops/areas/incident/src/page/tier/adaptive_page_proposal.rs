use super::*;

use acme_kit::text;

/// Where a adaptive page proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePageProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptivePageProposalStatus {
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

impl std::fmt::Display for AdaptivePageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive page proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageProposal {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub ratio: f64,
    pub revision: i64,
    pub status: AdaptivePageProposalStatus,
}

impl AdaptivePageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePageProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePageProposalStatus) -> bool {
        match self.status {
            AdaptivePageProposalStatus::Draft => {
                return next == AdaptivePageProposalStatus::Active || next == AdaptivePageProposalStatus::Closed;
            }
            AdaptivePageProposalStatus::Pending => {
                return next == AdaptivePageProposalStatus::Suspended;
            }
            AdaptivePageProposalStatus::Active => {
                match next {
                    AdaptivePageProposalStatus::Suspended | AdaptivePageProposalStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePageProposalStatus::Suspended => {
                match next {
                    AdaptivePageProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptivePageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive page proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ends_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePageProposal {
        AdaptivePageProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            ratio: self.ratio,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive page proposal in status draft.
pub fn new_adaptive_page_proposal(code: &str, label: &str) -> AdaptivePageProposal {
    AdaptivePageProposal {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        ratio: 0.0,
        revision: 0,
        status: AdaptivePageProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_page_proposal("code-468", "label-974");
        assert_eq!(item.status, AdaptivePageProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_adaptive_page_proposal("code-468", "label-974");
        if item.move_to(AdaptivePageProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePageProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_page_proposal("code-468", "label-974");
        item.status = AdaptivePageProposalStatus::Closed;
        assert!(
            item.move_to(AdaptivePageProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
