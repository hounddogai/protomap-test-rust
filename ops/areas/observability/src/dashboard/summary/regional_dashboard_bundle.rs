use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional dashboard bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDashboardBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalDashboardBundleStatus {
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

impl std::fmt::Display for RegionalDashboardBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional dashboard bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundle {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub expires_at: i64,
    pub issuer: String,
    pub score: f64,
    pub channel: String,
    pub timezone: String,
    pub priority: i64,
    pub status: RegionalDashboardBundleStatus,
}

impl RegionalDashboardBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDashboardBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDashboardBundleStatus) -> bool {
        match self.status {
            RegionalDashboardBundleStatus::Open => {
                return next == RegionalDashboardBundleStatus::InReview
                    || next == RegionalDashboardBundleStatus::Reopened;
            }
            RegionalDashboardBundleStatus::InReview => {
                match next {
                    RegionalDashboardBundleStatus::Resolved | RegionalDashboardBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDashboardBundleStatus::Resolved => {
                return next == RegionalDashboardBundleStatus::Reopened
                    || next == RegionalDashboardBundleStatus::Archived;
            }
            RegionalDashboardBundleStatus::Reopened => {
                return next == RegionalDashboardBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDashboardBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional dashboard bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDashboardBundle {
        RegionalDashboardBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            margin: self.margin,
            attributes: self.attributes.clone(),
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
            score: self.score,
            channel: self.channel.clone(),
            timezone: self.timezone.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional dashboard bundle in status open.
pub fn new_regional_dashboard_bundle(code: &str, name: &str) -> RegionalDashboardBundle {
    RegionalDashboardBundle {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        margin: 0.0,
        attributes: BTreeMap::new(),
        expires_at: 0,
        issuer: "".to_string(),
        score: 0.0,
        channel: "".to_string(),
        timezone: "".to_string(),
        priority: 0,
        status: RegionalDashboardBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_dashboard_bundle("code-8", "name-267");
        assert_eq!(item.status, RegionalDashboardBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_regional_dashboard_bundle("code-8", "name-267");
        if item.move_to(RegionalDashboardBundleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDashboardBundleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_dashboard_bundle("code-8", "name-267");
        item.status = RegionalDashboardBundleStatus::Archived;
        assert!(
            item.move_to(RegionalDashboardBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
