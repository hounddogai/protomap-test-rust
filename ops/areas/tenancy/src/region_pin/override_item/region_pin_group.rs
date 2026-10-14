use super::*;

use acme_kit::text;

/// Where a region pin group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionPinGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionPinGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin group as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroup {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub timezone: String,
    pub revision: i64,
    pub rate: f64,
    pub status: RegionPinGroupStatus,
}

impl RegionPinGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinGroupStatus) -> bool {
        match self.status {
            RegionPinGroupStatus::Proposed => {
                match next {
                    RegionPinGroupStatus::Approved | RegionPinGroupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinGroupStatus::Approved => {
                return next == RegionPinGroupStatus::Withdrawn;
            }
            RegionPinGroupStatus::Rejected => {
                match next {
                    RegionPinGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionPinGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.revision as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinGroup {
        RegionPinGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            timezone: self.timezone.clone(),
            revision: self.revision,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a region pin group in status proposed.
pub fn new_region_pin_group(code: &str, title: &str) -> RegionPinGroup {
    RegionPinGroup {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        timezone: "".to_string(),
        revision: 0,
        rate: 0.0,
        status: RegionPinGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_region_pin_group("code-288", "title-121");
        assert_eq!(item.status, RegionPinGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_region_pin_group("code-288", "title-121");
        if item.move_to(RegionPinGroupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinGroupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_region_pin_group("code-288", "title-121");
        item.status = RegionPinGroupStatus::Withdrawn;
        assert!(
            item.move_to(RegionPinGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
