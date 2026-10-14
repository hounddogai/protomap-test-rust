use super::*;

use acme_kit::text;

/// Where a legacy egress bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEgressBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyEgressBundleStatus {
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

impl std::fmt::Display for LegacyEgressBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy egress bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressBundle {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub description: String,
    pub sequence: i64,
    pub priority: i64,
    pub unit: String,
    pub owner_id: String,
    pub score: f64,
    pub currency: String,
    pub duration_minutes: i64,
    pub status: LegacyEgressBundleStatus,
}

impl LegacyEgressBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEgressBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEgressBundleStatus) -> bool {
        match self.status {
            LegacyEgressBundleStatus::Open => {
                match next {
                    LegacyEgressBundleStatus::InReview | LegacyEgressBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEgressBundleStatus::InReview => {
                match next {
                    LegacyEgressBundleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEgressBundleStatus::Resolved => {
                match next {
                    LegacyEgressBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEgressBundleStatus::Reopened => {
                return next == LegacyEgressBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEgressBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy egress bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        (self.score * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEgressBundle {
        LegacyEgressBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            description: self.description.clone(),
            sequence: self.sequence,
            priority: self.priority,
            unit: self.unit.clone(),
            owner_id: self.owner_id.clone(),
            score: self.score,
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy egress bundle in status open.
pub fn new_legacy_egress_bundle(code: &str, title: &str) -> LegacyEgressBundle {
    LegacyEgressBundle {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        description: "".to_string(),
        sequence: 0,
        priority: 0,
        unit: "".to_string(),
        owner_id: "".to_string(),
        score: 0.0,
        currency: "".to_string(),
        duration_minutes: 0,
        status: LegacyEgressBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_egress_bundle("code-84", "title-485");
        assert_eq!(item.status, LegacyEgressBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_legacy_egress_bundle("code-84", "title-485");
        if item.move_to(LegacyEgressBundleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyEgressBundleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_egress_bundle("code-84", "title-485");
        item.status = LegacyEgressBundleStatus::Archived;
        assert!(
            item.move_to(LegacyEgressBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
