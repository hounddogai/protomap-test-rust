use super::*;

use acme_kit::text;

/// Where a fallback health check rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackHealthCheckRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackHealthCheckRollupStatus {
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

impl std::fmt::Display for FallbackHealthCheckRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback health check rollup as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckRollup {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub maximum: f64,
    pub reviewer: String,
    pub principal: String,
    pub minimum: f64,
    pub summary: String,
    pub pinned: bool,
    pub cluster: String,
    pub discount: f64,
    pub status: FallbackHealthCheckRollupStatus,
}

impl FallbackHealthCheckRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackHealthCheckRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackHealthCheckRollupStatus) -> bool {
        match self.status {
            FallbackHealthCheckRollupStatus::Draft => {
                return next == FallbackHealthCheckRollupStatus::Active
                    || next == FallbackHealthCheckRollupStatus::Suspended;
            }
            FallbackHealthCheckRollupStatus::Pending => {
                return next == FallbackHealthCheckRollupStatus::Suspended;
            }
            FallbackHealthCheckRollupStatus::Active => {
                return next == FallbackHealthCheckRollupStatus::Suspended
                    || next == FallbackHealthCheckRollupStatus::Closed;
            }
            FallbackHealthCheckRollupStatus::Suspended => {
                return next == FallbackHealthCheckRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackHealthCheckRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback health check rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.maximum * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackHealthCheckRollup {
        FallbackHealthCheckRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            principal: self.principal.clone(),
            minimum: self.minimum,
            summary: self.summary.clone(),
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback health check rollup in status draft.
pub fn new_fallback_health_check_rollup(code: &str, name: &str) -> FallbackHealthCheckRollup {
    FallbackHealthCheckRollup {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        maximum: 0.0,
        reviewer: "".to_string(),
        principal: "".to_string(),
        minimum: 0.0,
        summary: "".to_string(),
        pinned: false,
        cluster: "".to_string(),
        discount: 0.0,
        status: FallbackHealthCheckRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_health_check_rollup("code-87", "name-23");
        assert_eq!(item.status, FallbackHealthCheckRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_fallback_health_check_rollup("code-87", "name-23");
        if item.move_to(FallbackHealthCheckRollupStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackHealthCheckRollupStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_health_check_rollup("code-87", "name-23");
        item.status = FallbackHealthCheckRollupStatus::Closed;
        assert!(
            item.move_to(FallbackHealthCheckRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
