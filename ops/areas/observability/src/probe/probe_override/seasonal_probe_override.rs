use super::*;

use acme_kit::text;

/// Where a seasonal probe override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalProbeOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalProbeOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalProbeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal probe override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeOverride {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub floor_value: f64,
    pub margin: f64,
    pub checksum: String,
    pub pinned: bool,
    pub active: bool,
    pub discount: f64,
    pub visible: bool,
    pub label: String,
    pub status: SeasonalProbeOverrideStatus,
}

impl SeasonalProbeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalProbeOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalProbeOverrideStatus) -> bool {
        match self.status {
            SeasonalProbeOverrideStatus::Requested => {
                return next == SeasonalProbeOverrideStatus::Expired;
            }
            SeasonalProbeOverrideStatus::Granted => {
                match next {
                    SeasonalProbeOverrideStatus::Revoked | SeasonalProbeOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalProbeOverrideStatus::Revoked => {
                return next == SeasonalProbeOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalProbeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal probe override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.margin - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalProbeOverride {
        SeasonalProbeOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            floor_value: self.floor_value,
            margin: self.margin,
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            active: self.active,
            discount: self.discount,
            visible: self.visible,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal probe override in status requested.
pub fn new_seasonal_probe_override(code: &str, name: &str) -> SeasonalProbeOverride {
    SeasonalProbeOverride {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        floor_value: 0.0,
        margin: 0.0,
        checksum: "".to_string(),
        pinned: false,
        active: false,
        discount: 0.0,
        visible: false,
        label: "".to_string(),
        status: SeasonalProbeOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_probe_override("code-97", "name-221");
        assert_eq!(item.status, SeasonalProbeOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_seasonal_probe_override("code-97", "name-221");
        if item.move_to(SeasonalProbeOverrideStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalProbeOverrideStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_probe_override("code-97", "name-221");
        item.status = SeasonalProbeOverrideStatus::Expired;
        assert!(
            item.move_to(SeasonalProbeOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
