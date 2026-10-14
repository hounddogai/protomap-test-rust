use super::*;

use acme_kit::text;

/// Where a federation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationTemplateStatus {
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

impl std::fmt::Display for FederationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationTemplate {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub summary: String,
    pub issuer: String,
    pub pinned: bool,
    pub amount: f64,
    pub status: FederationTemplateStatus,
}

impl FederationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationTemplateStatus) -> bool {
        match self.status {
            FederationTemplateStatus::Open => {
                return next == FederationTemplateStatus::Resolved;
            }
            FederationTemplateStatus::InReview => {
                return next == FederationTemplateStatus::Resolved || next == FederationTemplateStatus::Reopened;
            }
            FederationTemplateStatus::Resolved => {
                return next == FederationTemplateStatus::Reopened || next == FederationTemplateStatus::Archived;
            }
            FederationTemplateStatus::Reopened => {
                return next == FederationTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> FederationTemplate {
        FederationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            issuer: self.issuer.clone(),
            pinned: self.pinned,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation template in status open.
pub fn new_federation_template(code: &str, title: &str) -> FederationTemplate {
    FederationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        issuer: "".to_string(),
        pinned: false,
        amount: 0.0,
        status: FederationTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_template("code-583", "title-489");
        assert_eq!(item.status, FederationTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_federation_template("code-583", "title-489");
        if item.move_to(FederationTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_template("code-583", "title-489");
        item.status = FederationTemplateStatus::Archived;
        assert!(
            item.move_to(FederationTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
