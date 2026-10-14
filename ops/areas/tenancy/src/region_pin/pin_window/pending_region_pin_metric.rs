use super::*;

use acme_kit::text;

/// Where a pending region pin metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRegionPinMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingRegionPinMetricStatus {
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

impl std::fmt::Display for PendingRegionPinMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending region pin metric as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetric {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub grace_days: i64,
    pub locale: String,
    pub fingerprint: String,
    pub scope: String,
    pub label: String,
    pub tags: Vec<String>,
    pub cost: f64,
    pub status: PendingRegionPinMetricStatus,
}

impl PendingRegionPinMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRegionPinMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRegionPinMetricStatus) -> bool {
        match self.status {
            PendingRegionPinMetricStatus::Draft => {
                return next == PendingRegionPinMetricStatus::Active;
            }
            PendingRegionPinMetricStatus::Pending => {
                return next == PendingRegionPinMetricStatus::Suspended || next == PendingRegionPinMetricStatus::Closed;
            }
            PendingRegionPinMetricStatus::Active => {
                match next {
                    PendingRegionPinMetricStatus::Suspended | PendingRegionPinMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRegionPinMetricStatus::Suspended => {
                match next {
                    PendingRegionPinMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingRegionPinMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending region pin metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.cost + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRegionPinMetric {
        PendingRegionPinMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            fingerprint: self.fingerprint.clone(),
            scope: self.scope.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a pending region pin metric in status draft.
pub fn new_pending_region_pin_metric(code: &str, name: &str) -> PendingRegionPinMetric {
    PendingRegionPinMetric {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        grace_days: 0,
        locale: "".to_string(),
        fingerprint: "".to_string(),
        scope: "".to_string(),
        label: "".to_string(),
        tags: Vec::new(),
        cost: 0.0,
        status: PendingRegionPinMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_region_pin_metric("code-159", "name-484");
        assert_eq!(item.status, PendingRegionPinMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_region_pin_metric("code-159", "name-484");
        if item.move_to(PendingRegionPinMetricStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRegionPinMetricStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_region_pin_metric("code-159", "name-484");
        item.status = PendingRegionPinMetricStatus::Closed;
        assert!(
            item.move_to(PendingRegionPinMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
