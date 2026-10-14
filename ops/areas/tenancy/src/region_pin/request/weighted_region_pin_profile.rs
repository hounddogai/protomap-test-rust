use super::*;

use acme_kit::text;

/// Where a weighted region pin profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRegionPinProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedRegionPinProfileStatus {
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

impl std::fmt::Display for WeightedRegionPinProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted region pin profile as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinProfile {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub tags: Vec<String>,
    pub version: i64,
    pub minimum: f64,
    pub grace_days: i64,
    pub status: WeightedRegionPinProfileStatus,
}

impl WeightedRegionPinProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRegionPinProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRegionPinProfileStatus) -> bool {
        match self.status {
            WeightedRegionPinProfileStatus::Idle => {
                match next {
                    WeightedRegionPinProfileStatus::Warming | WeightedRegionPinProfileStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRegionPinProfileStatus::Warming => {
                match next {
                    WeightedRegionPinProfileStatus::Ready | WeightedRegionPinProfileStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRegionPinProfileStatus::Ready => {
                match next {
                    WeightedRegionPinProfileStatus::Draining | WeightedRegionPinProfileStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRegionPinProfileStatus::Draining => {
                match next {
                    WeightedRegionPinProfileStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedRegionPinProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted region pin profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.minimum - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRegionPinProfile {
        WeightedRegionPinProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            tags: self.tags.clone(),
            version: self.version,
            minimum: self.minimum,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted region pin profile in status idle.
pub fn new_weighted_region_pin_profile(code: &str, name: &str) -> WeightedRegionPinProfile {
    WeightedRegionPinProfile {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        tags: Vec::new(),
        version: 0,
        minimum: 0.0,
        grace_days: 0,
        status: WeightedRegionPinProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_region_pin_profile("code-571", "name-760");
        assert_eq!(item.status, WeightedRegionPinProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_region_pin_profile("code-571", "name-760");
        if item.move_to(WeightedRegionPinProfileStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRegionPinProfileStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_region_pin_profile("code-571", "name-760");
        item.status = WeightedRegionPinProfileStatus::Stopped;
        assert!(
            item.move_to(WeightedRegionPinProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
