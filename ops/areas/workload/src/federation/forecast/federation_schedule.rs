use super::*;

use acme_kit::text;

/// Where a federation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationScheduleStatus {
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

impl std::fmt::Display for FederationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation schedule as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationSchedule {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub issuer: String,
    pub title: String,
    pub batch_size: i64,
    pub quota_units: i64,
    pub status: FederationScheduleStatus,
}

impl FederationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationScheduleStatus) -> bool {
        match self.status {
            FederationScheduleStatus::Open => {
                match next {
                    FederationScheduleStatus::InReview | FederationScheduleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationScheduleStatus::InReview => {
                return next == FederationScheduleStatus::Resolved;
            }
            FederationScheduleStatus::Resolved => {
                return next == FederationScheduleStatus::Reopened || next == FederationScheduleStatus::Archived;
            }
            FederationScheduleStatus::Reopened => {
                return next == FederationScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "federation schedule", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quota_units as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationSchedule {
        FederationSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a federation schedule in status open.
pub fn new_federation_schedule(code: &str, label: &str) -> FederationSchedule {
    FederationSchedule {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        issuer: "".to_string(),
        title: "".to_string(),
        batch_size: 0,
        quota_units: 0,
        status: FederationScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_schedule("code-418", "label-841");
        assert_eq!(item.status, FederationScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_federation_schedule("code-418", "label-841");
        if item.move_to(FederationScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_schedule("code-418", "label-841");
        item.status = FederationScheduleStatus::Archived;
        assert!(
            item.move_to(FederationScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
