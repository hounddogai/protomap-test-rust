use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy rollout window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyRolloutWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LegacyRolloutWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout window as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutWindow {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub cluster: String,
    pub category: String,
    pub currency: String,
    pub source: String,
    pub region: String,
    pub status: LegacyRolloutWindowStatus,
}

impl LegacyRolloutWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutWindowStatus) -> bool {
        match self.status {
            LegacyRolloutWindowStatus::Draft => {
                match next {
                    LegacyRolloutWindowStatus::Pending | LegacyRolloutWindowStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutWindowStatus::Pending => {
                match next {
                    LegacyRolloutWindowStatus::Suspended | LegacyRolloutWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutWindowStatus::Active => {
                match next {
                    LegacyRolloutWindowStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutWindowStatus::Suspended => {
                return next == LegacyRolloutWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRolloutWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> LegacyRolloutWindow {
        LegacyRolloutWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            attributes: self.attributes.clone(),
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            currency: self.currency.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy rollout window in status draft.
pub fn new_legacy_rollout_window(code: &str, name: &str) -> LegacyRolloutWindow {
    LegacyRolloutWindow {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        attributes: BTreeMap::new(),
        cluster: "".to_string(),
        category: "".to_string(),
        currency: "".to_string(),
        source: "".to_string(),
        region: "".to_string(),
        status: LegacyRolloutWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_rollout_window("code-828", "name-869");
        assert_eq!(item.status, LegacyRolloutWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_rollout_window("code-828", "name-869");
        if item.move_to(LegacyRolloutWindowStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRolloutWindowStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_rollout_window("code-828", "name-869");
        item.status = LegacyRolloutWindowStatus::Closed;
        assert!(
            item.move_to(LegacyRolloutWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
