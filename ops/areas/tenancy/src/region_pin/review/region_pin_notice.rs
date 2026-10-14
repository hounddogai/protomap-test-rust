use super::*;

use acme_kit::text;

/// Where a region pin notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin notice as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinNotice {
    pub code: String,
    pub title: String,
    pub region: String,
    pub name: String,
    pub attempts: i64,
    pub updated_at: i64,
    pub country: String,
    pub status: RegionPinNoticeStatus,
}

impl RegionPinNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinNoticeStatus) -> bool {
        match self.status {
            RegionPinNoticeStatus::Created => {
                match next {
                    RegionPinNoticeStatus::Validated | RegionPinNoticeStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinNoticeStatus::Validated => {
                return next == RegionPinNoticeStatus::Published || next == RegionPinNoticeStatus::Retired;
            }
            RegionPinNoticeStatus::Published => {
                return next == RegionPinNoticeStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.updated_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinNotice {
        RegionPinNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            updated_at: self.updated_at,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a region pin notice in status created.
pub fn new_region_pin_notice(code: &str, title: &str) -> RegionPinNotice {
    RegionPinNotice {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        name: "".to_string(),
        attempts: 0,
        updated_at: 0,
        country: "".to_string(),
        status: RegionPinNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_region_pin_notice("code-777", "title-660");
        assert_eq!(item.status, RegionPinNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_region_pin_notice("code-777", "title-660");
        if item.move_to(RegionPinNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_region_pin_notice("code-777", "title-660");
        item.status = RegionPinNoticeStatus::Retired;
        assert!(
            item.move_to(RegionPinNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
