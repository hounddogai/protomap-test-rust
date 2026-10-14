use super::*;

use acme_kit::text;

/// Where a query variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for QueryVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryVariance {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub threshold: f64,
    pub expires_at: i64,
    pub version: i64,
    pub ceiling: f64,
    pub quantity: i64,
    pub status: QueryVarianceStatus,
}

impl QueryVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryVarianceStatus) -> bool {
        match self.status {
            QueryVarianceStatus::Draft => {
                return next == QueryVarianceStatus::Pending;
            }
            QueryVarianceStatus::Pending => {
                match next {
                    QueryVarianceStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryVarianceStatus::Active => {
                match next {
                    QueryVarianceStatus::Suspended | QueryVarianceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryVarianceStatus::Suspended => {
                return next == QueryVarianceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ceiling + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryVariance {
        QueryVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            threshold: self.threshold,
            expires_at: self.expires_at,
            version: self.version,
            ceiling: self.ceiling,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query variance in status draft.
pub fn new_query_variance(code: &str, label: &str) -> QueryVariance {
    QueryVariance {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        threshold: 0.0,
        expires_at: 0,
        version: 0,
        ceiling: 0.0,
        quantity: 0,
        status: QueryVarianceStatus::Draft,
    }
}
