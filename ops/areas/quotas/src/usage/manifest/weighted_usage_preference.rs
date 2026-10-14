use super::*;

use acme_kit::text;

/// Where a weighted usage preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedUsagePreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedUsagePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedUsagePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted usage preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreference {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub checksum: String,
    pub tenant_id: String,
    pub retries: i64,
    pub status: WeightedUsagePreferenceStatus,
}

impl WeightedUsagePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedUsagePreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedUsagePreferenceStatus) -> bool {
        match self.status {
            WeightedUsagePreferenceStatus::Created => {
                return next == WeightedUsagePreferenceStatus::Retired;
            }
            WeightedUsagePreferenceStatus::Validated => {
                return next == WeightedUsagePreferenceStatus::Published
                    || next == WeightedUsagePreferenceStatus::Retired;
            }
            WeightedUsagePreferenceStatus::Published => {
                return next == WeightedUsagePreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedUsagePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted usage preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted usage preference", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> WeightedUsagePreference {
        WeightedUsagePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted usage preference in status created.
pub fn new_weighted_usage_preference(code: &str, title: &str) -> WeightedUsagePreference {
    WeightedUsagePreference {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        retries: 0,
        status: WeightedUsagePreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_usage_preference("code-782", "title-648");
        assert_eq!(item.status, WeightedUsagePreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_weighted_usage_preference("code-782", "title-648");
        if item.move_to(WeightedUsagePreferenceStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedUsagePreferenceStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_usage_preference("code-782", "title-648");
        item.status = WeightedUsagePreferenceStatus::Retired;
        assert!(
            item.move_to(WeightedUsagePreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
