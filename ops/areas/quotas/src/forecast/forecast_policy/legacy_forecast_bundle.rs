use super::*;

use acme_kit::text;

/// Where a legacy forecast bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyForecastBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyForecastBundleStatus {
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

impl std::fmt::Display for LegacyForecastBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy forecast bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyForecastBundle {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub tags: Vec<String>,
    pub capacity: i64,
    pub currency: String,
    pub status: LegacyForecastBundleStatus,
}

impl LegacyForecastBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyForecastBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyForecastBundleStatus) -> bool {
        match self.status {
            LegacyForecastBundleStatus::Draft => {
                return next == LegacyForecastBundleStatus::Active || next == LegacyForecastBundleStatus::Closed;
            }
            LegacyForecastBundleStatus::Pending => {
                match next {
                    LegacyForecastBundleStatus::Suspended | LegacyForecastBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyForecastBundleStatus::Active => {
                return next == LegacyForecastBundleStatus::Suspended || next == LegacyForecastBundleStatus::Closed;
            }
            LegacyForecastBundleStatus::Suspended => {
                match next {
                    LegacyForecastBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyForecastBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy forecast bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.capacity as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyForecastBundle {
        LegacyForecastBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            tags: self.tags.clone(),
            capacity: self.capacity,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy forecast bundle in status draft.
pub fn new_legacy_forecast_bundle(code: &str, label: &str) -> LegacyForecastBundle {
    LegacyForecastBundle {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        tags: Vec::new(),
        capacity: 0,
        currency: "".to_string(),
        status: LegacyForecastBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_forecast_bundle("code-146", "label-267");
        assert_eq!(item.status, LegacyForecastBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_forecast_bundle("code-146", "label-267");
        if item.move_to(LegacyForecastBundleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyForecastBundleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_forecast_bundle("code-146", "label-267");
        item.status = LegacyForecastBundleStatus::Closed;
        assert!(
            item.move_to(LegacyForecastBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
