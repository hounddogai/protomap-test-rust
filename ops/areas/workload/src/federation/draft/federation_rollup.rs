use super::*;

use acme_kit::text;

/// Where a federation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationRollupStatus {
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

impl std::fmt::Display for FederationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation rollup as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationRollup {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub checksum: String,
    pub contact_email: String,
    pub active: bool,
    pub category: String,
    pub pinned: bool,
    pub status: FederationRollupStatus,
}

impl FederationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationRollupStatus) -> bool {
        match self.status {
            FederationRollupStatus::Open => {
                match next {
                    FederationRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRollupStatus::InReview => {
                match next {
                    FederationRollupStatus::Reopened | FederationRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRollupStatus::Resolved => {
                match next {
                    FederationRollupStatus::Reopened | FederationRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRollupStatus::Reopened => {
                return next == FederationRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> FederationRollup {
        FederationRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            active: self.active,
            category: self.category.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a federation rollup in status open.
pub fn new_federation_rollup(code: &str, label: &str) -> FederationRollup {
    FederationRollup {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        checksum: "".to_string(),
        contact_email: "".to_string(),
        active: false,
        category: "".to_string(),
        pinned: false,
        status: FederationRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_rollup("code-370", "label-132");
        assert_eq!(item.status, FederationRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_federation_rollup("code-370", "label-132");
        if item.move_to(FederationRollupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationRollupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_rollup("code-370", "label-132");
        item.status = FederationRollupStatus::Archived;
        assert!(
            item.move_to(FederationRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
