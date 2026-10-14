use super::*;

use acme_kit::text;

/// Where a manual query agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualQueryAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualQueryAgreementStatus {
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

impl std::fmt::Display for ManualQueryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual query agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQueryAgreement {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub duration_minutes: i64,
    pub fingerprint: String,
    pub label: String,
    pub visible: bool,
    pub cost: f64,
    pub retries: i64,
    pub owner_id: String,
    pub unit: String,
    pub status: ManualQueryAgreementStatus,
}

impl ManualQueryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualQueryAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualQueryAgreementStatus) -> bool {
        match self.status {
            ManualQueryAgreementStatus::Open => {
                return next == ManualQueryAgreementStatus::Reopened;
            }
            ManualQueryAgreementStatus::InReview => {
                match next {
                    ManualQueryAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQueryAgreementStatus::Resolved => {
                match next {
                    ManualQueryAgreementStatus::Reopened | ManualQueryAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQueryAgreementStatus::Reopened => {
                return next == ManualQueryAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualQueryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual query agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual query agreement", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.cost * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualQueryAgreement {
        ManualQueryAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            duration_minutes: self.duration_minutes,
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            visible: self.visible,
            cost: self.cost,
            retries: self.retries,
            owner_id: self.owner_id.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual query agreement in status open.
pub fn new_manual_query_agreement(code: &str, name: &str) -> ManualQueryAgreement {
    ManualQueryAgreement {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        duration_minutes: 0,
        fingerprint: "".to_string(),
        label: "".to_string(),
        visible: false,
        cost: 0.0,
        retries: 0,
        owner_id: "".to_string(),
        unit: "".to_string(),
        status: ManualQueryAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_query_agreement("code-607", "name-717");
        assert_eq!(item.status, ManualQueryAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_manual_query_agreement("code-607", "name-717");
        if item.move_to(ManualQueryAgreementStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualQueryAgreementStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_query_agreement("code-607", "name-717");
        item.status = ManualQueryAgreementStatus::Archived;
        assert!(
            item.move_to(ManualQueryAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
