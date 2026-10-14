use super::*;

use acme_kit::text;

/// Where a primary tenant score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTenantScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryTenantScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimaryTenantScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary tenant score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScore {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub quota_units: i64,
    pub cost: f64,
    pub status: PrimaryTenantScoreStatus,
}

impl PrimaryTenantScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTenantScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTenantScoreStatus) -> bool {
        match self.status {
            PrimaryTenantScoreStatus::New => {
                return next == PrimaryTenantScoreStatus::Queued;
            }
            PrimaryTenantScoreStatus::Queued => {
                match next {
                    PrimaryTenantScoreStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTenantScoreStatus::Running => {
                match next {
                    PrimaryTenantScoreStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTenantScoreStatus::Done => {
                return next == PrimaryTenantScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTenantScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary tenant score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary tenant score", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        (self.cost * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTenantScore {
        PrimaryTenantScore {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary tenant score in status new.
pub fn new_primary_tenant_score(code: &str, title: &str) -> PrimaryTenantScore {
    PrimaryTenantScore {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        cost: 0.0,
        status: PrimaryTenantScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_tenant_score("code-538", "title-615");
        assert_eq!(item.status, PrimaryTenantScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_primary_tenant_score("code-538", "title-615");
        if item.move_to(PrimaryTenantScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTenantScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_tenant_score("code-538", "title-615");
        item.status = PrimaryTenantScoreStatus::Failed;
        assert!(
            item.move_to(PrimaryTenantScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
