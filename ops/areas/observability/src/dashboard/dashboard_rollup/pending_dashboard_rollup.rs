use super::*;

use acme_kit::text;

/// Where a pending dashboard rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDashboardRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingDashboardRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingDashboardRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending dashboard rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollup {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub description: String,
    pub rate: f64,
    pub status: PendingDashboardRollupStatus,
}

impl PendingDashboardRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDashboardRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDashboardRollupStatus) -> bool {
        match self.status {
            PendingDashboardRollupStatus::Created => {
                match next {
                    PendingDashboardRollupStatus::Validated | PendingDashboardRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDashboardRollupStatus::Validated => {
                match next {
                    PendingDashboardRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDashboardRollupStatus::Published => {
                return next == PendingDashboardRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingDashboardRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending dashboard rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDashboardRollup {
        PendingDashboardRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            description: self.description.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending dashboard rollup in status created.
pub fn new_pending_dashboard_rollup(code: &str, title: &str) -> PendingDashboardRollup {
    PendingDashboardRollup {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        description: "".to_string(),
        rate: 0.0,
        status: PendingDashboardRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_dashboard_rollup("code-868", "title-736");
        assert_eq!(item.status, PendingDashboardRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_dashboard_rollup("code-868", "title-736");
        if item.move_to(PendingDashboardRollupStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDashboardRollupStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_dashboard_rollup("code-868", "title-736");
        item.status = PendingDashboardRollupStatus::Retired;
        assert!(
            item.move_to(PendingDashboardRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
