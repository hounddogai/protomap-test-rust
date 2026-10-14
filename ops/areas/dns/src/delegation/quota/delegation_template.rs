use super::*;

use acme_kit::text;

/// Where a delegation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTemplate {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub source: String,
    pub priority: i64,
    pub active: bool,
    pub status: DelegationTemplateStatus,
}

impl DelegationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationTemplateStatus) -> bool {
        match self.status {
            DelegationTemplateStatus::Proposed => {
                return next == DelegationTemplateStatus::Rejected || next == DelegationTemplateStatus::Withdrawn;
            }
            DelegationTemplateStatus::Approved => {
                return next == DelegationTemplateStatus::Rejected;
            }
            DelegationTemplateStatus::Rejected => {
                match next {
                    DelegationTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation template", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> DelegationTemplate {
        DelegationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            source: self.source.clone(),
            priority: self.priority,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a delegation template in status proposed.
pub fn new_delegation_template(code: &str, title: &str) -> DelegationTemplate {
    DelegationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        source: "".to_string(),
        priority: 0,
        active: false,
        status: DelegationTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_template("code-353", "title-780");
        assert_eq!(item.status, DelegationTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_delegation_template("code-353", "title-780");
        if item.move_to(DelegationTemplateStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationTemplateStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_template("code-353", "title-780");
        item.status = DelegationTemplateStatus::Withdrawn;
        assert!(
            item.move_to(DelegationTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
