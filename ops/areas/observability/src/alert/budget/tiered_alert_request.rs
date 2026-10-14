use super::*;

use acme_kit::text;

/// Where a tiered alert request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAlertRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredAlertRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredAlertRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered alert request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRequest {
    pub code: String,
    pub title: String,
    pub region: String,
    pub ratio: f64,
    pub country: String,
    pub created_at: i64,
    pub floor_value: f64,
    pub checksum: String,
    pub ceiling: f64,
    pub status: TieredAlertRequestStatus,
}

impl TieredAlertRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAlertRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAlertRequestStatus) -> bool {
        match self.status {
            TieredAlertRequestStatus::Requested => {
                return next == TieredAlertRequestStatus::Granted;
            }
            TieredAlertRequestStatus::Granted => {
                return next == TieredAlertRequestStatus::Expired;
            }
            TieredAlertRequestStatus::Revoked => {
                match next {
                    TieredAlertRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: TieredAlertRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered alert request {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAlertRequest {
        TieredAlertRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            ratio: self.ratio,
            country: self.country.clone(),
            created_at: self.created_at,
            floor_value: self.floor_value,
            checksum: self.checksum.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered alert request in status requested.
pub fn new_tiered_alert_request(code: &str, title: &str) -> TieredAlertRequest {
    TieredAlertRequest {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        ratio: 0.0,
        country: "".to_string(),
        created_at: 0,
        floor_value: 0.0,
        checksum: "".to_string(),
        ceiling: 0.0,
        status: TieredAlertRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_alert_request("code-720", "title-45");
        assert_eq!(item.status, TieredAlertRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_tiered_alert_request("code-720", "title-45");
        if item.move_to(TieredAlertRequestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAlertRequestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_alert_request("code-720", "title-45");
        item.status = TieredAlertRequestStatus::Expired;
        assert!(
            item.move_to(TieredAlertRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
