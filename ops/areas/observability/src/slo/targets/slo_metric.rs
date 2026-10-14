use super::*;

use acme_kit::text;

/// Where a slo metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SloMetricStatus {
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

impl std::fmt::Display for SloMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo metric as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloMetric {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub version: i64,
    pub created_at: i64,
    pub audience: String,
    pub phone: String,
    pub rate: f64,
    pub budget: f64,
    pub minimum: f64,
    pub status: SloMetricStatus,
}

impl SloMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloMetricStatus) -> bool {
        match self.status {
            SloMetricStatus::Draft => {
                match next {
                    SloMetricStatus::Pending | SloMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloMetricStatus::Pending => {
                match next {
                    SloMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloMetricStatus::Active => {
                return next == SloMetricStatus::Closed;
            }
            SloMetricStatus::Suspended => {
                return next == SloMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo metric {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rate - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloMetric {
        SloMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            version: self.version,
            created_at: self.created_at,
            audience: self.audience.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            budget: self.budget,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a slo metric in status draft.
pub fn new_slo_metric(code: &str, name: &str) -> SloMetric {
    SloMetric {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        version: 0,
        created_at: 0,
        audience: "".to_string(),
        phone: "".to_string(),
        rate: 0.0,
        budget: 0.0,
        minimum: 0.0,
        status: SloMetricStatus::Draft,
    }
}
