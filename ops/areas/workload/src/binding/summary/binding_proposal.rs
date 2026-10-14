use super::*;

use acme_kit::text;

/// Where a binding proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BindingProposalStatus {
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

impl std::fmt::Display for BindingProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposal {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub category: String,
    pub fingerprint: String,
    pub version: i64,
    pub batch_size: i64,
    pub quota_units: i64,
    pub status: BindingProposalStatus,
}

impl BindingProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingProposalStatus) -> bool {
        match self.status {
            BindingProposalStatus::Draft => {
                return next == BindingProposalStatus::Pending || next == BindingProposalStatus::Active;
            }
            BindingProposalStatus::Pending => {
                return next == BindingProposalStatus::Suspended || next == BindingProposalStatus::Closed;
            }
            BindingProposalStatus::Active => {
                return next == BindingProposalStatus::Suspended;
            }
            BindingProposalStatus::Suspended => {
                match next {
                    BindingProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: BindingProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingProposal {
        BindingProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            tags: self.tags.clone(),
            category: self.category.clone(),
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a binding proposal in status draft.
pub fn new_binding_proposal(code: &str, label: &str) -> BindingProposal {
    BindingProposal {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        tags: Vec::new(),
        category: "".to_string(),
        fingerprint: "".to_string(),
        version: 0,
        batch_size: 0,
        quota_units: 0,
        status: BindingProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_binding_proposal("code-761", "label-586");
        assert_eq!(item.status, BindingProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_binding_proposal("code-761", "label-586");
        if item.move_to(BindingProposalStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingProposalStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_binding_proposal("code-761", "label-586");
        item.status = BindingProposalStatus::Closed;
        assert!(
            item.move_to(BindingProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
