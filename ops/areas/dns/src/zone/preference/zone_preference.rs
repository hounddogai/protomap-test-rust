use super::*;

use acme_kit::text;

/// Where a zone preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZonePreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZonePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZonePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonePreference {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub ratio: f64,
    pub scope: String,
    pub host: String,
    pub burst: i64,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub unit: String,
    pub discount: f64,
    pub status: ZonePreferenceStatus,
}

impl ZonePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZonePreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZonePreferenceStatus) -> bool {
        match self.status {
            ZonePreferenceStatus::Created => {
                return next == ZonePreferenceStatus::Published;
            }
            ZonePreferenceStatus::Validated => {
                return next == ZonePreferenceStatus::Retired;
            }
            ZonePreferenceStatus::Published => {
                match next {
                    ZonePreferenceStatus::Retired => {
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

    pub fn move_to(&mut self, next: ZonePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "zone preference", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        (self.ratio * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZonePreference {
        ZonePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            ratio: self.ratio,
            scope: self.scope.clone(),
            host: self.host.clone(),
            burst: self.burst,
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            unit: self.unit.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a zone preference in status created.
pub fn new_zone_preference(code: &str, label: &str) -> ZonePreference {
    ZonePreference {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        ratio: 0.0,
        scope: "".to_string(),
        host: "".to_string(),
        burst: 0,
        maximum: 0.0,
        duration_minutes: 0,
        unit: "".to_string(),
        discount: 0.0,
        status: ZonePreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_zone_preference("code-44", "label-864");
        assert_eq!(item.status, ZonePreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_zone_preference("code-44", "label-864");
        if item.move_to(ZonePreferenceStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZonePreferenceStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_zone_preference("code-44", "label-864");
        item.status = ZonePreferenceStatus::Retired;
        assert!(
            item.move_to(ZonePreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
