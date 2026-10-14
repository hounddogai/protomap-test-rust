use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tenant review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantReview {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub locale: String,
    pub source: String,
    pub phone: String,
    pub category: String,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub status: TenantReviewStatus,
}

impl TenantReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantReviewStatus) -> bool {
        match self.status {
            TenantReviewStatus::Created => {
                return next == TenantReviewStatus::Published || next == TenantReviewStatus::Retired;
            }
            TenantReviewStatus::Validated => {
                match next {
                    TenantReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantReviewStatus::Published => {
                return next == TenantReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TenantReview {
        TenantReview {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            locale: self.locale.clone(),
            source: self.source.clone(),
            phone: self.phone.clone(),
            category: self.category.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tenant review in status created.
pub fn new_tenant_review(code: &str, name: &str) -> TenantReview {
    TenantReview {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        locale: "".to_string(),
        source: "".to_string(),
        phone: "".to_string(),
        category: "".to_string(),
        attributes: BTreeMap::new(),
        score: 0.0,
        status: TenantReviewStatus::Created,
    }
}
