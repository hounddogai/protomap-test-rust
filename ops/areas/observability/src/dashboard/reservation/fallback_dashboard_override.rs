use super::*;

use acme_kit::text;

/// Where a fallback dashboard override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDashboardOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackDashboardOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackDashboardOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback dashboard override as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardOverride {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub summary: String,
    pub retries: i64,
    pub visible: bool,
    pub minimum: f64,
    pub version: i64,
    pub status: FallbackDashboardOverrideStatus,
}

impl FallbackDashboardOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDashboardOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDashboardOverrideStatus) -> bool {
        match self.status {
            FallbackDashboardOverrideStatus::Created => {
                match next {
                    FallbackDashboardOverrideStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDashboardOverrideStatus::Validated => {
                return next == FallbackDashboardOverrideStatus::Published;
            }
            FallbackDashboardOverrideStatus::Published => {
                return next == FallbackDashboardOverrideStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackDashboardOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback dashboard override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.retries as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDashboardOverride {
        FallbackDashboardOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            summary: self.summary.clone(),
            retries: self.retries,
            visible: self.visible,
            minimum: self.minimum,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback dashboard override in status created.
pub fn new_fallback_dashboard_override(code: &str, title: &str) -> FallbackDashboardOverride {
    FallbackDashboardOverride {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        summary: "".to_string(),
        retries: 0,
        visible: false,
        minimum: 0.0,
        version: 0,
        status: FallbackDashboardOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_dashboard_override("code-960", "title-950");
        assert_eq!(item.status, FallbackDashboardOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_fallback_dashboard_override("code-960", "title-950");
        if item.move_to(FallbackDashboardOverrideStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDashboardOverrideStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_dashboard_override("code-960", "title-950");
        item.status = FallbackDashboardOverrideStatus::Retired;
        assert!(
            item.move_to(FallbackDashboardOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
