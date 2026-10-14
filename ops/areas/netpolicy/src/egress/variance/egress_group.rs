use super::*;

use acme_kit::text;

/// Where a egress group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EgressGroupStatus {
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

impl std::fmt::Display for EgressGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroup {
    pub code: String,
    pub label: String,
    pub host: String,
    pub maximum: f64,
    pub notes: String,
    pub contact_email: String,
    pub external_ref: String,
    pub quota_units: i64,
    pub score: f64,
    pub pinned: bool,
    pub status: EgressGroupStatus,
}

impl EgressGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressGroupStatus) -> bool {
        match self.status {
            EgressGroupStatus::Open => {
                return next == EgressGroupStatus::InReview;
            }
            EgressGroupStatus::InReview => {
                return next == EgressGroupStatus::Resolved || next == EgressGroupStatus::Archived;
            }
            EgressGroupStatus::Resolved => {
                return next == EgressGroupStatus::Reopened;
            }
            EgressGroupStatus::Reopened => {
                return next == EgressGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress group", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.maximum * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressGroup {
        EgressGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            maximum: self.maximum,
            notes: self.notes.clone(),
            contact_email: self.contact_email.clone(),
            external_ref: self.external_ref.clone(),
            quota_units: self.quota_units,
            score: self.score,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a egress group in status open.
pub fn new_egress_group(code: &str, label: &str) -> EgressGroup {
    EgressGroup {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        maximum: 0.0,
        notes: "".to_string(),
        contact_email: "".to_string(),
        external_ref: "".to_string(),
        quota_units: 0,
        score: 0.0,
        pinned: false,
        status: EgressGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_egress_group("code-580", "label-246");
        assert_eq!(item.status, EgressGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_egress_group("code-580", "label-246");
        if item.move_to(EgressGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_egress_group("code-580", "label-246");
        item.status = EgressGroupStatus::Archived;
        assert!(item.move_to(EgressGroupStatus::Open).is_err(), "expected move to fail");
    }
}
