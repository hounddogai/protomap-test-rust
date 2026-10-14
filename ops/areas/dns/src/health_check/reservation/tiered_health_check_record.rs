use super::*;

use acme_kit::text;

/// Where a tiered health check record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredHealthCheckRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredHealthCheckRecordStatus {
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

impl std::fmt::Display for TieredHealthCheckRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered health check record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRecord {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub maximum: f64,
    pub summary: String,
    pub ttl_seconds: i64,
    pub phone: String,
    pub budget: f64,
    pub country: String,
    pub ends_at: i64,
    pub status: TieredHealthCheckRecordStatus,
}

impl TieredHealthCheckRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredHealthCheckRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredHealthCheckRecordStatus) -> bool {
        match self.status {
            TieredHealthCheckRecordStatus::Draft => {
                match next {
                    TieredHealthCheckRecordStatus::Pending | TieredHealthCheckRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredHealthCheckRecordStatus::Pending => {
                return next == TieredHealthCheckRecordStatus::Closed;
            }
            TieredHealthCheckRecordStatus::Active => {
                match next {
                    TieredHealthCheckRecordStatus::Suspended | TieredHealthCheckRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredHealthCheckRecordStatus::Suspended => {
                match next {
                    TieredHealthCheckRecordStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredHealthCheckRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered health check record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered health check record", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ends_at as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredHealthCheckRecord {
        TieredHealthCheckRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            summary: self.summary.clone(),
            ttl_seconds: self.ttl_seconds,
            phone: self.phone.clone(),
            budget: self.budget,
            country: self.country.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered health check record in status draft.
pub fn new_tiered_health_check_record(code: &str, title: &str) -> TieredHealthCheckRecord {
    TieredHealthCheckRecord {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        maximum: 0.0,
        summary: "".to_string(),
        ttl_seconds: 0,
        phone: "".to_string(),
        budget: 0.0,
        country: "".to_string(),
        ends_at: 0,
        status: TieredHealthCheckRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_health_check_record("code-400", "title-460");
        assert_eq!(item.status, TieredHealthCheckRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tiered_health_check_record("code-400", "title-460");
        if item.move_to(TieredHealthCheckRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredHealthCheckRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_health_check_record("code-400", "title-460");
        item.status = TieredHealthCheckRecordStatus::Closed;
        assert!(
            item.move_to(TieredHealthCheckRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
