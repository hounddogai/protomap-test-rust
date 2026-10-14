use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a zone rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ZoneRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ZoneRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRollup {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub phone: String,
    pub audience: String,
    pub cost: f64,
    pub ratio: f64,
    pub status: ZoneRollupStatus,
}

impl ZoneRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneRollupStatus) -> bool {
        match self.status {
            ZoneRollupStatus::Requested => {
                return next == ZoneRollupStatus::Expired;
            }
            ZoneRollupStatus::Granted => {
                return next == ZoneRollupStatus::Revoked || next == ZoneRollupStatus::Expired;
            }
            ZoneRollupStatus::Revoked => {
                return next == ZoneRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn weighted_volume(&self) -> f64 {
        (self.ratio * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneRollup {
        ZoneRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            phone: self.phone.clone(),
            audience: self.audience.clone(),
            cost: self.cost,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone rollup in status requested.
pub fn new_zone_rollup(code: &str, title: &str) -> ZoneRollup {
    ZoneRollup {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        phone: "".to_string(),
        audience: "".to_string(),
        cost: 0.0,
        ratio: 0.0,
        status: ZoneRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_zone_rollup("code-861", "title-564");
        assert_eq!(item.status, ZoneRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_zone_rollup("code-861", "title-564");
        if item.move_to(ZoneRollupStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneRollupStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_zone_rollup("code-861", "title-564");
        item.status = ZoneRollupStatus::Expired;
        assert!(
            item.move_to(ZoneRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
