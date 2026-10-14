use super::*;

use acme_kit::text;

/// Where a tiered health check variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredHealthCheckVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredHealthCheckVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredHealthCheckVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered health check variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckVariance {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub created_at: i64,
    pub issuer: String,
    pub key_id: String,
    pub archived: bool,
    pub attempts: i64,
    pub margin: f64,
    pub burst: i64,
    pub status: TieredHealthCheckVarianceStatus,
}

impl TieredHealthCheckVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredHealthCheckVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredHealthCheckVarianceStatus) -> bool {
        match self.status {
            TieredHealthCheckVarianceStatus::Created => {
                return next == TieredHealthCheckVarianceStatus::Validated;
            }
            TieredHealthCheckVarianceStatus::Validated => {
                return next == TieredHealthCheckVarianceStatus::Published
                    || next == TieredHealthCheckVarianceStatus::Retired;
            }
            TieredHealthCheckVarianceStatus::Published => {
                return next == TieredHealthCheckVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredHealthCheckVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered health check variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered health check variance", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.margin + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredHealthCheckVariance {
        TieredHealthCheckVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            archived: self.archived,
            attempts: self.attempts,
            margin: self.margin,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered health check variance in status created.
pub fn new_tiered_health_check_variance(code: &str, label: &str) -> TieredHealthCheckVariance {
    TieredHealthCheckVariance {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        created_at: 0,
        issuer: "".to_string(),
        key_id: "".to_string(),
        archived: false,
        attempts: 0,
        margin: 0.0,
        burst: 0,
        status: TieredHealthCheckVarianceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_health_check_variance("code-470", "label-160");
        assert_eq!(item.status, TieredHealthCheckVarianceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_health_check_variance("code-470", "label-160");
        if item.move_to(TieredHealthCheckVarianceStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredHealthCheckVarianceStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_health_check_variance("code-470", "label-160");
        item.status = TieredHealthCheckVarianceStatus::Retired;
        assert!(
            item.move_to(TieredHealthCheckVarianceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
