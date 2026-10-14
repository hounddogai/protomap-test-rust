use super::*;

use acme_kit::text;

/// Where a regional region pin request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRegionPinRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalRegionPinRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalRegionPinRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional region pin request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRegionPinRequest {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub burst: i64,
    pub active: bool,
    pub rotation_days: i64,
    pub name: String,
    pub version: i64,
    pub limit: i64,
    pub quantity: i64,
    pub grace_days: i64,
    pub status: RegionalRegionPinRequestStatus,
}

impl RegionalRegionPinRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRegionPinRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRegionPinRequestStatus) -> bool {
        match self.status {
            RegionalRegionPinRequestStatus::Requested => {
                match next {
                    RegionalRegionPinRequestStatus::Revoked | RegionalRegionPinRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRegionPinRequestStatus::Granted => {
                match next {
                    RegionalRegionPinRequestStatus::Revoked | RegionalRegionPinRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRegionPinRequestStatus::Revoked => {
                match next {
                    RegionalRegionPinRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalRegionPinRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional region pin request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional region pin request", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRegionPinRequest {
        RegionalRegionPinRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            burst: self.burst,
            active: self.active,
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            version: self.version,
            limit: self.limit,
            quantity: self.quantity,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional region pin request in status requested.
pub fn new_regional_region_pin_request(code: &str, title: &str) -> RegionalRegionPinRequest {
    RegionalRegionPinRequest {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        burst: 0,
        active: false,
        rotation_days: 0,
        name: "".to_string(),
        version: 0,
        limit: 0,
        quantity: 0,
        grace_days: 0,
        status: RegionalRegionPinRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_region_pin_request("code-986", "title-576");
        assert_eq!(item.status, RegionalRegionPinRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_regional_region_pin_request("code-986", "title-576");
        if item.move_to(RegionalRegionPinRequestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRegionPinRequestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_region_pin_request("code-986", "title-576");
        item.status = RegionalRegionPinRequestStatus::Expired;
        assert!(
            item.move_to(RegionalRegionPinRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
