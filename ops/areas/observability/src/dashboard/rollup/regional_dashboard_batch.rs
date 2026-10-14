use super::*;

use acme_kit::text;

/// Where a regional dashboard batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDashboardBatchStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalDashboardBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalDashboardBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional dashboard batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBatch {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub category: String,
    pub expires_at: i64,
    pub summary: String,
    pub starts_at: i64,
    pub audience: String,
    pub weight: f64,
    pub maximum: f64,
    pub ratio: f64,
    pub status: RegionalDashboardBatchStatus,
}

impl RegionalDashboardBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDashboardBatchStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDashboardBatchStatus) -> bool {
        match self.status {
            RegionalDashboardBatchStatus::Created => {
                return next == RegionalDashboardBatchStatus::Published
                    || next == RegionalDashboardBatchStatus::Retired;
            }
            RegionalDashboardBatchStatus::Validated => {
                match next {
                    RegionalDashboardBatchStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDashboardBatchStatus::Published => {
                return next == RegionalDashboardBatchStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDashboardBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional dashboard batch {} cannot move from {} to {}",
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
        format!("{} {}", "regional dashboard batch", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.starts_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDashboardBatch {
        RegionalDashboardBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            category: self.category.clone(),
            expires_at: self.expires_at,
            summary: self.summary.clone(),
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            weight: self.weight,
            maximum: self.maximum,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional dashboard batch in status created.
pub fn new_regional_dashboard_batch(code: &str, title: &str) -> RegionalDashboardBatch {
    RegionalDashboardBatch {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        category: "".to_string(),
        expires_at: 0,
        summary: "".to_string(),
        starts_at: 0,
        audience: "".to_string(),
        weight: 0.0,
        maximum: 0.0,
        ratio: 0.0,
        status: RegionalDashboardBatchStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_dashboard_batch("code-903", "title-104");
        assert_eq!(item.status, RegionalDashboardBatchStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_regional_dashboard_batch("code-903", "title-104");
        if item.move_to(RegionalDashboardBatchStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDashboardBatchStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_dashboard_batch("code-903", "title-104");
        item.status = RegionalDashboardBatchStatus::Retired;
        assert!(
            item.move_to(RegionalDashboardBatchStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
