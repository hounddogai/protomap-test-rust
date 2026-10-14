use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a query adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QueryAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QueryAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAdjustment {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub reviewer: String,
    pub attributes: BTreeMap<String, String>,
    pub limit: i64,
    pub rate: f64,
    pub checksum: String,
    pub discount: f64,
    pub principal: String,
    pub starts_at: i64,
    pub status: QueryAdjustmentStatus,
}

impl QueryAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryAdjustmentStatus) -> bool {
        match self.status {
            QueryAdjustmentStatus::Created => {
                match next {
                    QueryAdjustmentStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryAdjustmentStatus::Validated => {
                match next {
                    QueryAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryAdjustmentStatus::Published => {
                return next == QueryAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
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

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryAdjustment {
        QueryAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            reviewer: self.reviewer.clone(),
            attributes: self.attributes.clone(),
            limit: self.limit,
            rate: self.rate,
            checksum: self.checksum.clone(),
            discount: self.discount,
            principal: self.principal.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query adjustment in status created.
pub fn new_query_adjustment(code: &str, title: &str) -> QueryAdjustment {
    QueryAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        reviewer: "".to_string(),
        attributes: BTreeMap::new(),
        limit: 0,
        rate: 0.0,
        checksum: "".to_string(),
        discount: 0.0,
        principal: "".to_string(),
        starts_at: 0,
        status: QueryAdjustmentStatus::Created,
    }
}
