use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a region pin override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionPinOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionPinOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinOverride {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub approved: bool,
    pub notes: String,
    pub status: RegionPinOverrideStatus,
}

impl RegionPinOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinOverrideStatus) -> bool {
        match self.status {
            RegionPinOverrideStatus::Requested => {
                match next {
                    RegionPinOverrideStatus::Granted | RegionPinOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinOverrideStatus::Granted => {
                return next == RegionPinOverrideStatus::Revoked;
            }
            RegionPinOverrideStatus::Revoked => {
                match next {
                    RegionPinOverrideStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionPinOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin override", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> RegionPinOverride {
        RegionPinOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            approved: self.approved,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a region pin override in status requested.
pub fn new_region_pin_override(code: &str, name: &str) -> RegionPinOverride {
    RegionPinOverride {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        approved: false,
        notes: "".to_string(),
        status: RegionPinOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_region_pin_override("code-911", "name-938");
        assert_eq!(item.status, RegionPinOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_region_pin_override("code-911", "name-938");
        if item.move_to(RegionPinOverrideStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinOverrideStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_region_pin_override("code-911", "name-938");
        item.status = RegionPinOverrideStatus::Expired;
        assert!(
            item.move_to(RegionPinOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
