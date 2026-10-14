use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a postmortem variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PostmortemVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PostmortemVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem variance as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVariance {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub name: String,
    pub category: String,
    pub summary: String,
    pub locale: String,
    pub amount: f64,
    pub limit: i64,
    pub tenant_id: String,
    pub rate: f64,
    pub status: PostmortemVarianceStatus,
}

impl PostmortemVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemVarianceStatus) -> bool {
        match self.status {
            PostmortemVarianceStatus::Planned => {
                match next {
                    PostmortemVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemVarianceStatus::Scheduled => {
                return next == PostmortemVarianceStatus::Cancelled;
            }
            PostmortemVarianceStatus::InProgress => {
                match next {
                    PostmortemVarianceStatus::Completed | PostmortemVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemVarianceStatus::Completed => {
                match next {
                    PostmortemVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PostmortemVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem variance", self.code)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemVariance {
        PostmortemVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            summary: self.summary.clone(),
            locale: self.locale.clone(),
            amount: self.amount,
            limit: self.limit,
            tenant_id: self.tenant_id.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a postmortem variance in status planned.
pub fn new_postmortem_variance(code: &str, label: &str) -> PostmortemVariance {
    PostmortemVariance {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        name: "".to_string(),
        category: "".to_string(),
        summary: "".to_string(),
        locale: "".to_string(),
        amount: 0.0,
        limit: 0,
        tenant_id: "".to_string(),
        rate: 0.0,
        status: PostmortemVarianceStatus::Planned,
    }
}
