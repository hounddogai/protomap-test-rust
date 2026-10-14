use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional page profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPageProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalPageProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalPageProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional page profile as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageProfile {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub grace_days: i64,
    pub cost: f64,
    pub key_id: String,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub tenant_id: String,
    pub quantity: i64,
    pub expires_at: i64,
    pub status: RegionalPageProfileStatus,
}

impl RegionalPageProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPageProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPageProfileStatus) -> bool {
        match self.status {
            RegionalPageProfileStatus::Created => {
                return next == RegionalPageProfileStatus::Validated || next == RegionalPageProfileStatus::Retired;
            }
            RegionalPageProfileStatus::Validated => {
                return next == RegionalPageProfileStatus::Published || next == RegionalPageProfileStatus::Retired;
            }
            RegionalPageProfileStatus::Published => {
                return next == RegionalPageProfileStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPageProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional page profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
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

    pub fn adjusted_load(&self) -> f64 {
        (self.maximum * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPageProfile {
        RegionalPageProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            grace_days: self.grace_days,
            cost: self.cost,
            key_id: self.key_id.clone(),
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional page profile in status created.
pub fn new_regional_page_profile(code: &str, title: &str) -> RegionalPageProfile {
    RegionalPageProfile {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        grace_days: 0,
        cost: 0.0,
        key_id: "".to_string(),
        attributes: BTreeMap::new(),
        window_days: 0,
        tenant_id: "".to_string(),
        quantity: 0,
        expires_at: 0,
        status: RegionalPageProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_page_profile("code-439", "title-309");
        assert_eq!(item.status, RegionalPageProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_regional_page_profile("code-439", "title-309");
        if item.move_to(RegionalPageProfileStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPageProfileStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_page_profile("code-439", "title-309");
        item.status = RegionalPageProfileStatus::Retired;
        assert!(
            item.move_to(RegionalPageProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
