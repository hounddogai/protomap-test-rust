use super::*;

use acme_kit::text;

/// Where a identity baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl IdentityBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for IdentityBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity baseline as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaseline {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub segment: String,
    pub locale: String,
    pub cluster: String,
    pub country: String,
    pub contact_email: String,
    pub approved: bool,
    pub issuer: String,
    pub score: f64,
    pub status: IdentityBaselineStatus,
}

impl IdentityBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityBaselineStatus) -> bool {
        match self.status {
            IdentityBaselineStatus::Proposed => {
                match next {
                    IdentityBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityBaselineStatus::Approved => {
                match next {
                    IdentityBaselineStatus::Rejected | IdentityBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityBaselineStatus::Rejected => {
                return next == IdentityBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> IdentityBaseline {
        IdentityBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            locale: self.locale.clone(),
            cluster: self.cluster.clone(),
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            approved: self.approved,
            issuer: self.issuer.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a identity baseline in status proposed.
pub fn new_identity_baseline(code: &str, label: &str) -> IdentityBaseline {
    IdentityBaseline {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        segment: "".to_string(),
        locale: "".to_string(),
        cluster: "".to_string(),
        country: "".to_string(),
        contact_email: "".to_string(),
        approved: false,
        issuer: "".to_string(),
        score: 0.0,
        status: IdentityBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_identity_baseline("code-952", "label-742");
        assert_eq!(item.status, IdentityBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_identity_baseline("code-952", "label-742");
        if item.move_to(IdentityBaselineStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityBaselineStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_identity_baseline("code-952", "label-742");
        item.status = IdentityBaselineStatus::Withdrawn;
        assert!(
            item.move_to(IdentityBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
