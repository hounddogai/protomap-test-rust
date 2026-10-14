use super::*;

use acme_kit::text;

/// Where a regional suspension profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSuspensionProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalSuspensionProfileStatus {
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

impl std::fmt::Display for RegionalSuspensionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional suspension profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfile {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub rotation_days: i64,
    pub limit: i64,
    pub floor_value: f64,
    pub score: f64,
    pub margin: f64,
    pub attempts: i64,
    pub cluster: String,
    pub status: RegionalSuspensionProfileStatus,
}

impl RegionalSuspensionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSuspensionProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSuspensionProfileStatus) -> bool {
        match self.status {
            RegionalSuspensionProfileStatus::Open => {
                match next {
                    RegionalSuspensionProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSuspensionProfileStatus::InReview => {
                return next == RegionalSuspensionProfileStatus::Resolved
                    || next == RegionalSuspensionProfileStatus::Reopened;
            }
            RegionalSuspensionProfileStatus::Resolved => {
                return next == RegionalSuspensionProfileStatus::Archived;
            }
            RegionalSuspensionProfileStatus::Reopened => {
                return next == RegionalSuspensionProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSuspensionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional suspension profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.floor_value * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSuspensionProfile {
        RegionalSuspensionProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            rotation_days: self.rotation_days,
            limit: self.limit,
            floor_value: self.floor_value,
            score: self.score,
            margin: self.margin,
            attempts: self.attempts,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a regional suspension profile in status open.
pub fn new_regional_suspension_profile(code: &str, title: &str) -> RegionalSuspensionProfile {
    RegionalSuspensionProfile {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        rotation_days: 0,
        limit: 0,
        floor_value: 0.0,
        score: 0.0,
        margin: 0.0,
        attempts: 0,
        cluster: "".to_string(),
        status: RegionalSuspensionProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_suspension_profile("code-887", "title-425");
        assert_eq!(item.status, RegionalSuspensionProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_regional_suspension_profile("code-887", "title-425");
        if item.move_to(RegionalSuspensionProfileStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSuspensionProfileStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_suspension_profile("code-887", "title-425");
        item.status = RegionalSuspensionProfileStatus::Archived;
        assert!(
            item.move_to(RegionalSuspensionProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
