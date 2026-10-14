use super::*;

use acme_kit::text;

/// Where a scheduled release agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledReleaseAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledReleaseAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledReleaseAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled release agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseAgreement {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub external_ref: String,
    pub window_days: i64,
    pub principal: String,
    pub unit: String,
    pub status: ScheduledReleaseAgreementStatus,
}

impl ScheduledReleaseAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledReleaseAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledReleaseAgreementStatus) -> bool {
        match self.status {
            ScheduledReleaseAgreementStatus::Proposed => {
                return next == ScheduledReleaseAgreementStatus::Approved;
            }
            ScheduledReleaseAgreementStatus::Approved => {
                match next {
                    ScheduledReleaseAgreementStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReleaseAgreementStatus::Rejected => {
                return next == ScheduledReleaseAgreementStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledReleaseAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled release agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledReleaseAgreement {
        ScheduledReleaseAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            window_days: self.window_days,
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled release agreement in status proposed.
pub fn new_scheduled_release_agreement(code: &str, label: &str) -> ScheduledReleaseAgreement {
    ScheduledReleaseAgreement {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        external_ref: "".to_string(),
        window_days: 0,
        principal: "".to_string(),
        unit: "".to_string(),
        status: ScheduledReleaseAgreementStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_release_agreement("code-213", "label-293");
        assert_eq!(item.status, ScheduledReleaseAgreementStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_release_agreement("code-213", "label-293");
        if item.move_to(ScheduledReleaseAgreementStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledReleaseAgreementStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_release_agreement("code-213", "label-293");
        item.status = ScheduledReleaseAgreementStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledReleaseAgreementStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
