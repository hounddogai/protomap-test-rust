use super::*;

use acme_kit::text;

/// Where a release rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReleaseRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReleaseRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release rollup as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRollup {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub attempts: i64,
    pub port: i64,
    pub margin: f64,
    pub status: ReleaseRollupStatus,
}

impl ReleaseRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseRollupStatus) -> bool {
        match self.status {
            ReleaseRollupStatus::Proposed => {
                return next == ReleaseRollupStatus::Rejected || next == ReleaseRollupStatus::Withdrawn;
            }
            ReleaseRollupStatus::Approved => {
                match next {
                    ReleaseRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseRollupStatus::Rejected => {
                match next {
                    ReleaseRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ReleaseRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.port as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseRollup {
        ReleaseRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            attempts: self.attempts,
            port: self.port,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release rollup in status proposed.
pub fn new_release_rollup(code: &str, name: &str) -> ReleaseRollup {
    ReleaseRollup {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        attempts: 0,
        port: 0,
        margin: 0.0,
        status: ReleaseRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_release_rollup("code-242", "name-47");
        assert_eq!(item.status, ReleaseRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_release_rollup("code-242", "name-47");
        if item.move_to(ReleaseRollupStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseRollupStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_release_rollup("code-242", "name-47");
        item.status = ReleaseRollupStatus::Withdrawn;
        assert!(
            item.move_to(ReleaseRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
