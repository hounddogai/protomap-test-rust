use super::*;

use acme_kit::text;

/// Where a region pin request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionPinRequestStatus {
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

impl std::fmt::Display for RegionPinRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRequest {
    pub code: String,
    pub name: String,
    pub country: String,
    pub title: String,
    pub archived: bool,
    pub created_at: i64,
    pub quantity: i64,
    pub host: String,
    pub status: RegionPinRequestStatus,
}

impl RegionPinRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinRequestStatus) -> bool {
        match self.status {
            RegionPinRequestStatus::Draft => {
                return next == RegionPinRequestStatus::Pending;
            }
            RegionPinRequestStatus::Pending => {
                match next {
                    RegionPinRequestStatus::Active | RegionPinRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinRequestStatus::Active => {
                match next {
                    RegionPinRequestStatus::Suspended | RegionPinRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinRequestStatus::Suspended => {
                return next == RegionPinRequestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin request {} cannot move from {} to {}",
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
        format!("{} {}", "region pin request", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quantity as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinRequest {
        RegionPinRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            title: self.title.clone(),
            archived: self.archived,
            created_at: self.created_at,
            quantity: self.quantity,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a region pin request in status draft.
pub fn new_region_pin_request(code: &str, name: &str) -> RegionPinRequest {
    RegionPinRequest {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        title: "".to_string(),
        archived: false,
        created_at: 0,
        quantity: 0,
        host: "".to_string(),
        status: RegionPinRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_region_pin_request("code-225", "name-482");
        assert_eq!(item.status, RegionPinRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_region_pin_request("code-225", "name-482");
        if item.move_to(RegionPinRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_region_pin_request("code-225", "name-482");
        item.status = RegionPinRequestStatus::Closed;
        assert!(
            item.move_to(RegionPinRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
