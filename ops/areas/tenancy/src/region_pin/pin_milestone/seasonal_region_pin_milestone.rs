use super::*;

use acme_kit::text;

/// Where a seasonal region pin milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRegionPinMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalRegionPinMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalRegionPinMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal region pin milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinMilestone {
    pub code: String,
    pub title: String,
    pub host: String,
    pub pinned: bool,
    pub description: String,
    pub checksum: String,
    pub name: String,
    pub weight: f64,
    pub label: String,
    pub issuer: String,
    pub timezone: String,
    pub status: SeasonalRegionPinMilestoneStatus,
}

impl SeasonalRegionPinMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRegionPinMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRegionPinMilestoneStatus) -> bool {
        match self.status {
            SeasonalRegionPinMilestoneStatus::Requested => {
                match next {
                    SeasonalRegionPinMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRegionPinMilestoneStatus::Granted => {
                match next {
                    SeasonalRegionPinMilestoneStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRegionPinMilestoneStatus::Revoked => {
                return next == SeasonalRegionPinMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRegionPinMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal region pin milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal region pin milestone", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> SeasonalRegionPinMilestone {
        SeasonalRegionPinMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            pinned: self.pinned,
            description: self.description.clone(),
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            weight: self.weight,
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal region pin milestone in status requested.
pub fn new_seasonal_region_pin_milestone(code: &str, title: &str) -> SeasonalRegionPinMilestone {
    SeasonalRegionPinMilestone {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        pinned: false,
        description: "".to_string(),
        checksum: "".to_string(),
        name: "".to_string(),
        weight: 0.0,
        label: "".to_string(),
        issuer: "".to_string(),
        timezone: "".to_string(),
        status: SeasonalRegionPinMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_region_pin_milestone("code-52", "title-791");
        assert_eq!(item.status, SeasonalRegionPinMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_seasonal_region_pin_milestone("code-52", "title-791");
        if item.move_to(SeasonalRegionPinMilestoneStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRegionPinMilestoneStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_region_pin_milestone("code-52", "title-791");
        item.status = SeasonalRegionPinMilestoneStatus::Expired;
        assert!(
            item.move_to(SeasonalRegionPinMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
