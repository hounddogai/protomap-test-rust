use super::*;

use acme_kit::text;

/// Where a approval forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ApprovalForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalForecast {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub created_at: i64,
    pub rotation_days: i64,
    pub batch_size: i64,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub locale: String,
    pub minimum: f64,
    pub cipher: String,
    pub status: ApprovalForecastStatus,
}

impl ApprovalForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalForecastStatus) -> bool {
        match self.status {
            ApprovalForecastStatus::Idle => {
                return next == ApprovalForecastStatus::Draining || next == ApprovalForecastStatus::Stopped;
            }
            ApprovalForecastStatus::Warming => {
                return next == ApprovalForecastStatus::Ready || next == ApprovalForecastStatus::Draining;
            }
            ApprovalForecastStatus::Ready => {
                return next == ApprovalForecastStatus::Draining || next == ApprovalForecastStatus::Stopped;
            }
            ApprovalForecastStatus::Draining => {
                return next == ApprovalForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval forecast {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.created_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalForecast {
        ApprovalForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a approval forecast in status idle.
pub fn new_approval_forecast(code: &str, title: &str) -> ApprovalForecast {
    ApprovalForecast {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        created_at: 0,
        rotation_days: 0,
        batch_size: 0,
        quota_units: 0,
        tags: Vec::new(),
        locale: "".to_string(),
        minimum: 0.0,
        cipher: "".to_string(),
        status: ApprovalForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_approval_forecast("code-16", "title-541");
        assert_eq!(item.status, ApprovalForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_approval_forecast("code-16", "title-541");
        if item.move_to(ApprovalForecastStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalForecastStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_approval_forecast("code-16", "title-541");
        item.status = ApprovalForecastStatus::Stopped;
        assert!(
            item.move_to(ApprovalForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
