use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted page adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPageAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedPageAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedPageAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted page adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageAdjustment {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub limit: i64,
    pub active: bool,
    pub status: WeightedPageAdjustmentStatus,
}

impl WeightedPageAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPageAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPageAdjustmentStatus) -> bool {
        match self.status {
            WeightedPageAdjustmentStatus::Created => {
                match next {
                    WeightedPageAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPageAdjustmentStatus::Validated => {
                match next {
                    WeightedPageAdjustmentStatus::Published | WeightedPageAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPageAdjustmentStatus::Published => {
                return next == WeightedPageAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedPageAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted page adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn projected_load(&self) -> f64 {
        ((self.limit as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPageAdjustment {
        WeightedPageAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            attributes: self.attributes.clone(),
            limit: self.limit,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted page adjustment in status created.
pub fn new_weighted_page_adjustment(code: &str, label: &str) -> WeightedPageAdjustment {
    WeightedPageAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        attributes: BTreeMap::new(),
        limit: 0,
        active: false,
        status: WeightedPageAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_page_adjustment("code-234", "label-285");
        assert_eq!(item.status, WeightedPageAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_weighted_page_adjustment("code-234", "label-285");
        if item.move_to(WeightedPageAdjustmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPageAdjustmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_page_adjustment("code-234", "label-285");
        item.status = WeightedPageAdjustmentStatus::Retired;
        assert!(
            item.move_to(WeightedPageAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
