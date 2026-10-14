use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PlanManifestStatus {
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

impl std::fmt::Display for PlanManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanManifest {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub region: String,
    pub quota_units: i64,
    pub cost: f64,
    pub port: i64,
    pub status: PlanManifestStatus,
}

impl PlanManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanManifestStatus) -> bool {
        match self.status {
            PlanManifestStatus::Open => {
                match next {
                    PlanManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanManifestStatus::InReview => {
                match next {
                    PlanManifestStatus::Resolved | PlanManifestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanManifestStatus::Resolved => {
                match next {
                    PlanManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanManifestStatus::Reopened => {
                return next == PlanManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.budget - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanManifest {
        PlanManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            budget: self.budget,
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            region: self.region.clone(),
            quota_units: self.quota_units,
            cost: self.cost,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a plan manifest in status open.
pub fn new_plan_manifest(code: &str, title: &str) -> PlanManifest {
    PlanManifest {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        budget: 0.0,
        attributes: BTreeMap::new(),
        category: "".to_string(),
        region: "".to_string(),
        quota_units: 0,
        cost: 0.0,
        port: 0,
        status: PlanManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_plan_manifest("code-241", "title-533");
        assert_eq!(item.status, PlanManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_plan_manifest("code-241", "title-533");
        if item.move_to(PlanManifestStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanManifestStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_plan_manifest("code-241", "title-533");
        item.status = PlanManifestStatus::Archived;
        assert!(item.move_to(PlanManifestStatus::Open).is_err(), "expected move to fail");
    }
}
