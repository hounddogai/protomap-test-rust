use super::*;

use acme_kit::text;

/// Where a health check summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl HealthCheckSummaryStatus {
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

impl std::fmt::Display for HealthCheckSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSummary {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub revision: i64,
    pub port: i64,
    pub issuer: String,
    pub minimum: f64,
    pub locale: String,
    pub pinned: bool,
    pub status: HealthCheckSummaryStatus,
}

impl HealthCheckSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckSummaryStatus) -> bool {
        match self.status {
            HealthCheckSummaryStatus::Draft => {
                match next {
                    HealthCheckSummaryStatus::Pending | HealthCheckSummaryStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckSummaryStatus::Pending => {
                return next == HealthCheckSummaryStatus::Active || next == HealthCheckSummaryStatus::Closed;
            }
            HealthCheckSummaryStatus::Active => {
                return next == HealthCheckSummaryStatus::Suspended || next == HealthCheckSummaryStatus::Closed;
            }
            HealthCheckSummaryStatus::Suspended => {
                return next == HealthCheckSummaryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.port as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckSummary {
        HealthCheckSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            revision: self.revision,
            port: self.port,
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            locale: self.locale.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a health check summary in status draft.
pub fn new_health_check_summary(code: &str, title: &str) -> HealthCheckSummary {
    HealthCheckSummary {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        revision: 0,
        port: 0,
        issuer: "".to_string(),
        minimum: 0.0,
        locale: "".to_string(),
        pinned: false,
        status: HealthCheckSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_health_check_summary("code-47", "title-172");
        assert_eq!(item.status, HealthCheckSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_health_check_summary("code-47", "title-172");
        if item.move_to(HealthCheckSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_health_check_summary("code-47", "title-172");
        item.status = HealthCheckSummaryStatus::Closed;
        assert!(
            item.move_to(HealthCheckSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
