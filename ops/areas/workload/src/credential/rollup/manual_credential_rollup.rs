use super::*;

use acme_kit::text;

/// Where a manual credential rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCredentialRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualCredentialRollupStatus {
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

impl std::fmt::Display for ManualCredentialRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual credential rollup as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialRollup {
    pub code: String,
    pub title: String,
    pub name: String,
    pub description: String,
    pub segment: String,
    pub cost: f64,
    pub window_days: i64,
    pub pinned: bool,
    pub channel: String,
    pub capacity: i64,
    pub unit: String,
    pub status: ManualCredentialRollupStatus,
}

impl ManualCredentialRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCredentialRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCredentialRollupStatus) -> bool {
        match self.status {
            ManualCredentialRollupStatus::Open => {
                return next == ManualCredentialRollupStatus::InReview;
            }
            ManualCredentialRollupStatus::InReview => {
                return next == ManualCredentialRollupStatus::Reopened
                    || next == ManualCredentialRollupStatus::Archived;
            }
            ManualCredentialRollupStatus::Resolved => {
                return next == ManualCredentialRollupStatus::Reopened;
            }
            ManualCredentialRollupStatus::Reopened => {
                match next {
                    ManualCredentialRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualCredentialRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual credential rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.window_days as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCredentialRollup {
        ManualCredentialRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            cost: self.cost,
            window_days: self.window_days,
            pinned: self.pinned,
            channel: self.channel.clone(),
            capacity: self.capacity,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual credential rollup in status open.
pub fn new_manual_credential_rollup(code: &str, title: &str) -> ManualCredentialRollup {
    ManualCredentialRollup {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        cost: 0.0,
        window_days: 0,
        pinned: false,
        channel: "".to_string(),
        capacity: 0,
        unit: "".to_string(),
        status: ManualCredentialRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_credential_rollup("code-995", "title-932");
        assert_eq!(item.status, ManualCredentialRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_credential_rollup("code-995", "title-932");
        if item.move_to(ManualCredentialRollupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCredentialRollupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_credential_rollup("code-995", "title-932");
        item.status = ManualCredentialRollupStatus::Archived;
        assert!(
            item.move_to(ManualCredentialRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
