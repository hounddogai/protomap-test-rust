use super::*;

use acme_kit::text;

/// Where a region pin preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPreference {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub principal: String,
    pub name: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub status: RegionPinPreferenceStatus,
}

impl RegionPinPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinPreferenceStatus) -> bool {
        match self.status {
            RegionPinPreferenceStatus::Created => {
                return next == RegionPinPreferenceStatus::Validated || next == RegionPinPreferenceStatus::Published;
            }
            RegionPinPreferenceStatus::Validated => {
                return next == RegionPinPreferenceStatus::Retired;
            }
            RegionPinPreferenceStatus::Published => {
                return next == RegionPinPreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin preference {} cannot move from {} to {}",
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
        format!("{} {}", "region pin preference", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> RegionPinPreference {
        RegionPinPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a region pin preference in status created.
pub fn new_region_pin_preference(code: &str, title: &str) -> RegionPinPreference {
    RegionPinPreference {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        principal: "".to_string(),
        name: "".to_string(),
        tags: Vec::new(),
        created_at: 0,
        status: RegionPinPreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_region_pin_preference("code-263", "title-158");
        assert_eq!(item.status, RegionPinPreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_region_pin_preference("code-263", "title-158");
        if item.move_to(RegionPinPreferenceStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinPreferenceStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_region_pin_preference("code-263", "title-158");
        item.status = RegionPinPreferenceStatus::Retired;
        assert!(
            item.move_to(RegionPinPreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
