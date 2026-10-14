use super::*;

use acme_kit::text;

/// Where a weighted federation bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFederationBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedFederationBundleStatus {
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

impl std::fmt::Display for WeightedFederationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted federation bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundle {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub updated_at: i64,
    pub description: String,
    pub category: String,
    pub phone: String,
    pub rate: f64,
    pub ratio: f64,
    pub owner_id: String,
    pub attempts: i64,
    pub status: WeightedFederationBundleStatus,
}

impl WeightedFederationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFederationBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFederationBundleStatus) -> bool {
        match self.status {
            WeightedFederationBundleStatus::Open => {
                match next {
                    WeightedFederationBundleStatus::InReview | WeightedFederationBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationBundleStatus::InReview => {
                return next == WeightedFederationBundleStatus::Reopened;
            }
            WeightedFederationBundleStatus::Resolved => {
                match next {
                    WeightedFederationBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationBundleStatus::Reopened => {
                return next == WeightedFederationBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFederationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted federation bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.rate * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFederationBundle {
        WeightedFederationBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            updated_at: self.updated_at,
            description: self.description.clone(),
            category: self.category.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            ratio: self.ratio,
            owner_id: self.owner_id.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a weighted federation bundle in status open.
pub fn new_weighted_federation_bundle(code: &str, name: &str) -> WeightedFederationBundle {
    WeightedFederationBundle {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        updated_at: 0,
        description: "".to_string(),
        category: "".to_string(),
        phone: "".to_string(),
        rate: 0.0,
        ratio: 0.0,
        owner_id: "".to_string(),
        attempts: 0,
        status: WeightedFederationBundleStatus::Open,
    }
}
