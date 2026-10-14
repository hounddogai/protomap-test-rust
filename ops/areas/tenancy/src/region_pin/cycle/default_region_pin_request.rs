use super::*;

use acme_kit::text;

/// Where a default region pin request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRegionPinRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultRegionPinRequestStatus {
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

impl std::fmt::Display for DefaultRegionPinRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default region pin request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRegionPinRequest {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub phone: String,
    pub rate: f64,
    pub description: String,
    pub status: DefaultRegionPinRequestStatus,
}

impl DefaultRegionPinRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRegionPinRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRegionPinRequestStatus) -> bool {
        match self.status {
            DefaultRegionPinRequestStatus::Draft => {
                return next == DefaultRegionPinRequestStatus::Pending || next == DefaultRegionPinRequestStatus::Closed;
            }
            DefaultRegionPinRequestStatus::Pending => {
                match next {
                    DefaultRegionPinRequestStatus::Active | DefaultRegionPinRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRegionPinRequestStatus::Active => {
                return next == DefaultRegionPinRequestStatus::Suspended;
            }
            DefaultRegionPinRequestStatus::Suspended => {
                match next {
                    DefaultRegionPinRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultRegionPinRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default region pin request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> DefaultRegionPinRequest {
        DefaultRegionPinRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default region pin request in status draft.
pub fn new_default_region_pin_request(code: &str, name: &str) -> DefaultRegionPinRequest {
    DefaultRegionPinRequest {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        phone: "".to_string(),
        rate: 0.0,
        description: "".to_string(),
        status: DefaultRegionPinRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_region_pin_request("code-993", "name-548");
        assert_eq!(item.status, DefaultRegionPinRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_default_region_pin_request("code-993", "name-548");
        if item.move_to(DefaultRegionPinRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRegionPinRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_region_pin_request("code-993", "name-548");
        item.status = DefaultRegionPinRequestStatus::Closed;
        assert!(
            item.move_to(DefaultRegionPinRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
