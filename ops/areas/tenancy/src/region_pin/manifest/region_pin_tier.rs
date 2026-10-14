use super::*;

use acme_kit::text;

/// Where a region pin tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionPinTierStatus {
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

impl std::fmt::Display for RegionPinTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTier {
    pub code: String,
    pub label: String,
    pub country: String,
    pub maximum: f64,
    pub audience: String,
    pub approved: bool,
    pub title: String,
    pub key_id: String,
    pub status: RegionPinTierStatus,
}

impl RegionPinTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinTierStatus) -> bool {
        match self.status {
            RegionPinTierStatus::Idle => {
                match next {
                    RegionPinTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinTierStatus::Warming => {
                match next {
                    RegionPinTierStatus::Draining | RegionPinTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinTierStatus::Ready => {
                return next == RegionPinTierStatus::Draining;
            }
            RegionPinTierStatus::Draining => {
                match next {
                    RegionPinTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionPinTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "region pin tier", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> RegionPinTier {
        RegionPinTier {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            maximum: self.maximum,
            audience: self.audience.clone(),
            approved: self.approved,
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a region pin tier in status idle.
pub fn new_region_pin_tier(code: &str, label: &str) -> RegionPinTier {
    RegionPinTier {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        maximum: 0.0,
        audience: "".to_string(),
        approved: false,
        title: "".to_string(),
        key_id: "".to_string(),
        status: RegionPinTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_region_pin_tier("code-517", "label-685");
        assert_eq!(item.status, RegionPinTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_region_pin_tier("code-517", "label-685");
        if item.move_to(RegionPinTierStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinTierStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_region_pin_tier("code-517", "label-685");
        item.status = RegionPinTierStatus::Stopped;
        assert!(
            item.move_to(RegionPinTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
