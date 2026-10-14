use super::*;

use acme_kit::text;

/// Where a manual suspension quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSuspensionQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualSuspensionQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualSuspensionQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual suspension quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSuspensionQuota {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub rotation_days: i64,
    pub margin: f64,
    pub key_id: String,
    pub maximum: f64,
    pub tags: Vec<String>,
    pub namespace: String,
    pub ends_at: i64,
    pub status: ManualSuspensionQuotaStatus,
}

impl ManualSuspensionQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSuspensionQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSuspensionQuotaStatus) -> bool {
        match self.status {
            ManualSuspensionQuotaStatus::Created => {
                match next {
                    ManualSuspensionQuotaStatus::Validated | ManualSuspensionQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSuspensionQuotaStatus::Validated => {
                return next == ManualSuspensionQuotaStatus::Published || next == ManualSuspensionQuotaStatus::Retired;
            }
            ManualSuspensionQuotaStatus::Published => {
                match next {
                    ManualSuspensionQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualSuspensionQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual suspension quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual suspension quota", self.code)
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
        ((self.maximum + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSuspensionQuota {
        ManualSuspensionQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            rotation_days: self.rotation_days,
            margin: self.margin,
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual suspension quota in status created.
pub fn new_manual_suspension_quota(code: &str, name: &str) -> ManualSuspensionQuota {
    ManualSuspensionQuota {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        rotation_days: 0,
        margin: 0.0,
        key_id: "".to_string(),
        maximum: 0.0,
        tags: Vec::new(),
        namespace: "".to_string(),
        ends_at: 0,
        status: ManualSuspensionQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_suspension_quota("code-771", "name-907");
        assert_eq!(item.status, ManualSuspensionQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_suspension_quota("code-771", "name-907");
        if item.move_to(ManualSuspensionQuotaStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSuspensionQuotaStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_suspension_quota("code-771", "name-907");
        item.status = ManualSuspensionQuotaStatus::Retired;
        assert!(
            item.move_to(ManualSuspensionQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
