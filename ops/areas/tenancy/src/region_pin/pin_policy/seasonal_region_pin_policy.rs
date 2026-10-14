use super::*;

use acme_kit::text;

/// Where a seasonal region pin policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRegionPinPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalRegionPinPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalRegionPinPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal region pin policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinPolicy {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub principal: String,
    pub tags: Vec<String>,
    pub burst: i64,
    pub title: String,
    pub phone: String,
    pub status: SeasonalRegionPinPolicyStatus,
}

impl SeasonalRegionPinPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRegionPinPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRegionPinPolicyStatus) -> bool {
        match self.status {
            SeasonalRegionPinPolicyStatus::Created => {
                return next == SeasonalRegionPinPolicyStatus::Published
                    || next == SeasonalRegionPinPolicyStatus::Retired;
            }
            SeasonalRegionPinPolicyStatus::Validated => {
                return next == SeasonalRegionPinPolicyStatus::Published
                    || next == SeasonalRegionPinPolicyStatus::Retired;
            }
            SeasonalRegionPinPolicyStatus::Published => {
                return next == SeasonalRegionPinPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRegionPinPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal region pin policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal region pin policy", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.burst as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRegionPinPolicy {
        SeasonalRegionPinPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            principal: self.principal.clone(),
            tags: self.tags.clone(),
            burst: self.burst,
            title: self.title.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal region pin policy in status created.
pub fn new_seasonal_region_pin_policy(code: &str, name: &str) -> SeasonalRegionPinPolicy {
    SeasonalRegionPinPolicy {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        principal: "".to_string(),
        tags: Vec::new(),
        burst: 0,
        title: "".to_string(),
        phone: "".to_string(),
        status: SeasonalRegionPinPolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_seasonal_region_pin_policy("code-345", "name-315");
        assert_eq!(item.status, SeasonalRegionPinPolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_seasonal_region_pin_policy("code-345", "name-315");
        if item.move_to(SeasonalRegionPinPolicyStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRegionPinPolicyStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_seasonal_region_pin_policy("code-345", "name-315");
        item.status = SeasonalRegionPinPolicyStatus::Retired;
        assert!(
            item.move_to(SeasonalRegionPinPolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
