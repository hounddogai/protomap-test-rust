use super::*;

use acme_kit::text;

/// Where a freeze preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezePreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FreezePreferenceStatus {
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

impl std::fmt::Display for FreezePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezePreference {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub title: String,
    pub discount: f64,
    pub capacity: i64,
    pub starts_at: i64,
    pub channel: String,
    pub audience: String,
    pub status: FreezePreferenceStatus,
}

impl FreezePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezePreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezePreferenceStatus) -> bool {
        match self.status {
            FreezePreferenceStatus::Draft => {
                match next {
                    FreezePreferenceStatus::Pending | FreezePreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezePreferenceStatus::Pending => {
                match next {
                    FreezePreferenceStatus::Active | FreezePreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezePreferenceStatus::Active => {
                match next {
                    FreezePreferenceStatus::Suspended | FreezePreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezePreferenceStatus::Suspended => {
                match next {
                    FreezePreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: FreezePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze preference {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "freeze preference", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezePreference {
        FreezePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            title: self.title.clone(),
            discount: self.discount,
            capacity: self.capacity,
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a freeze preference in status draft.
pub fn new_freeze_preference(code: &str, label: &str) -> FreezePreference {
    FreezePreference {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        title: "".to_string(),
        discount: 0.0,
        capacity: 0,
        starts_at: 0,
        channel: "".to_string(),
        audience: "".to_string(),
        status: FreezePreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_freeze_preference("code-205", "label-75");
        assert_eq!(item.status, FreezePreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_freeze_preference("code-205", "label-75");
        if item.move_to(FreezePreferenceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezePreferenceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_freeze_preference("code-205", "label-75");
        item.status = FreezePreferenceStatus::Closed;
        assert!(
            item.move_to(FreezePreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
