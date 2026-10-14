use super::*;

use acme_kit::text;

/// Where a delegation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DelegationGroupStatus {
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

impl std::fmt::Display for DelegationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation group as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationGroup {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub phone: String,
    pub principal: String,
    pub score: f64,
    pub status: DelegationGroupStatus,
}

impl DelegationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationGroupStatus) -> bool {
        match self.status {
            DelegationGroupStatus::Open => {
                match next {
                    DelegationGroupStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationGroupStatus::InReview => {
                match next {
                    DelegationGroupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationGroupStatus::Resolved => {
                return next == DelegationGroupStatus::Reopened || next == DelegationGroupStatus::Archived;
            }
            DelegationGroupStatus::Reopened => {
                return next == DelegationGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> DelegationGroup {
        DelegationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            phone: self.phone.clone(),
            principal: self.principal.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation group in status open.
pub fn new_delegation_group(code: &str, label: &str) -> DelegationGroup {
    DelegationGroup {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        phone: "".to_string(),
        principal: "".to_string(),
        score: 0.0,
        status: DelegationGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_delegation_group("code-365", "label-623");
        assert_eq!(item.status, DelegationGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_delegation_group("code-365", "label-623");
        if item.move_to(DelegationGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_delegation_group("code-365", "label-623");
        item.status = DelegationGroupStatus::Archived;
        assert!(
            item.move_to(DelegationGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
