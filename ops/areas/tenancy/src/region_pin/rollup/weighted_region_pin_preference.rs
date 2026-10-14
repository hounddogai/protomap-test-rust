use super::*;

use acme_kit::text;

/// Where a weighted region pin preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRegionPinPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedRegionPinPreferenceStatus {
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

impl std::fmt::Display for WeightedRegionPinPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted region pin preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinPreference {
    pub code: String,
    pub title: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub visible: bool,
    pub sequence: i64,
    pub maximum: f64,
    pub discount: f64,
    pub port: i64,
    pub tenant_id: String,
    pub status: WeightedRegionPinPreferenceStatus,
}

impl WeightedRegionPinPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRegionPinPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRegionPinPreferenceStatus) -> bool {
        match self.status {
            WeightedRegionPinPreferenceStatus::Draft => {
                match next {
                    WeightedRegionPinPreferenceStatus::Pending | WeightedRegionPinPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRegionPinPreferenceStatus::Pending => {
                match next {
                    WeightedRegionPinPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRegionPinPreferenceStatus::Active => {
                return next == WeightedRegionPinPreferenceStatus::Suspended
                    || next == WeightedRegionPinPreferenceStatus::Closed;
            }
            WeightedRegionPinPreferenceStatus::Suspended => {
                return next == WeightedRegionPinPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRegionPinPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted region pin preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.discount + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRegionPinPreference {
        WeightedRegionPinPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            visible: self.visible,
            sequence: self.sequence,
            maximum: self.maximum,
            discount: self.discount,
            port: self.port,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted region pin preference in status draft.
pub fn new_weighted_region_pin_preference(code: &str, title: &str) -> WeightedRegionPinPreference {
    WeightedRegionPinPreference {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        ttl_seconds: 0,
        visible: false,
        sequence: 0,
        maximum: 0.0,
        discount: 0.0,
        port: 0,
        tenant_id: "".to_string(),
        status: WeightedRegionPinPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_region_pin_preference("code-426", "title-350");
        assert_eq!(item.status, WeightedRegionPinPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_weighted_region_pin_preference("code-426", "title-350");
        if item.move_to(WeightedRegionPinPreferenceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRegionPinPreferenceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_region_pin_preference("code-426", "title-350");
        item.status = WeightedRegionPinPreferenceStatus::Closed;
        assert!(
            item.move_to(WeightedRegionPinPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
