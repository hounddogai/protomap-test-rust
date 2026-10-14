use super::*;

use acme_kit::text;

/// Where a page tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageTierStatus {
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

impl std::fmt::Display for PageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page tier as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTier {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub capacity: i64,
    pub owner_id: String,
    pub description: String,
    pub expires_at: i64,
    pub updated_at: i64,
    pub cost: f64,
    pub status: PageTierStatus,
}

impl PageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageTierStatus) -> bool {
        match self.status {
            PageTierStatus::Open => {
                match next {
                    PageTierStatus::Resolved | PageTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageTierStatus::InReview => {
                return next == PageTierStatus::Resolved;
            }
            PageTierStatus::Resolved => {
                match next {
                    PageTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageTierStatus::Reopened => {
                return next == PageTierStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page tier {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.expires_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageTier {
        PageTier {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            capacity: self.capacity,
            owner_id: self.owner_id.clone(),
            description: self.description.clone(),
            expires_at: self.expires_at,
            updated_at: self.updated_at,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page tier in status open.
pub fn new_page_tier(code: &str, name: &str) -> PageTier {
    PageTier {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        capacity: 0,
        owner_id: "".to_string(),
        description: "".to_string(),
        expires_at: 0,
        updated_at: 0,
        cost: 0.0,
        status: PageTierStatus::Open,
    }
}
