use super::*;

use acme_kit::text;

/// Where a regional allowlist target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalAllowlistTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalAllowlistTargetStatus {
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

impl std::fmt::Display for RegionalAllowlistTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional allowlist target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTarget {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub revision: i64,
    pub rotation_days: i64,
    pub minimum: f64,
    pub owner_id: String,
    pub host: String,
    pub issuer: String,
    pub capacity: i64,
    pub status: RegionalAllowlistTargetStatus,
}

impl RegionalAllowlistTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalAllowlistTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalAllowlistTargetStatus) -> bool {
        match self.status {
            RegionalAllowlistTargetStatus::Open => {
                return next == RegionalAllowlistTargetStatus::InReview
                    || next == RegionalAllowlistTargetStatus::Reopened;
            }
            RegionalAllowlistTargetStatus::InReview => {
                match next {
                    RegionalAllowlistTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalAllowlistTargetStatus::Resolved => {
                return next == RegionalAllowlistTargetStatus::Reopened;
            }
            RegionalAllowlistTargetStatus::Reopened => {
                match next {
                    RegionalAllowlistTargetStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalAllowlistTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional allowlist target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional allowlist target", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalAllowlistTarget {
        RegionalAllowlistTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            revision: self.revision,
            rotation_days: self.rotation_days,
            minimum: self.minimum,
            owner_id: self.owner_id.clone(),
            host: self.host.clone(),
            issuer: self.issuer.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional allowlist target in status open.
pub fn new_regional_allowlist_target(code: &str, label: &str) -> RegionalAllowlistTarget {
    RegionalAllowlistTarget {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        revision: 0,
        rotation_days: 0,
        minimum: 0.0,
        owner_id: "".to_string(),
        host: "".to_string(),
        issuer: "".to_string(),
        capacity: 0,
        status: RegionalAllowlistTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_allowlist_target("code-679", "label-905");
        assert_eq!(item.status, RegionalAllowlistTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_regional_allowlist_target("code-679", "label-905");
        if item.move_to(RegionalAllowlistTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalAllowlistTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_allowlist_target("code-679", "label-905");
        item.status = RegionalAllowlistTargetStatus::Archived;
        assert!(
            item.move_to(RegionalAllowlistTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
