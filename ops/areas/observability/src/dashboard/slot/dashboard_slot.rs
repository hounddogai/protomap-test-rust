use super::*;

use acme_kit::text;

/// Where a dashboard slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DashboardSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DashboardSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard slot as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSlot {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub title: String,
    pub limit: i64,
    pub ends_at: i64,
    pub region: String,
    pub summary: String,
    pub margin: f64,
    pub country: String,
    pub floor_value: f64,
    pub status: DashboardSlotStatus,
}

impl DashboardSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardSlotStatus) -> bool {
        match self.status {
            DashboardSlotStatus::Proposed => {
                match next {
                    DashboardSlotStatus::Approved | DashboardSlotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardSlotStatus::Approved => {
                match next {
                    DashboardSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardSlotStatus::Rejected => {
                match next {
                    DashboardSlotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DashboardSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard slot {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.floor_value - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardSlot {
        DashboardSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            title: self.title.clone(),
            limit: self.limit,
            ends_at: self.ends_at,
            region: self.region.clone(),
            summary: self.summary.clone(),
            margin: self.margin,
            country: self.country.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a dashboard slot in status proposed.
pub fn new_dashboard_slot(code: &str, label: &str) -> DashboardSlot {
    DashboardSlot {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        title: "".to_string(),
        limit: 0,
        ends_at: 0,
        region: "".to_string(),
        summary: "".to_string(),
        margin: 0.0,
        country: "".to_string(),
        floor_value: 0.0,
        status: DashboardSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_dashboard_slot("code-483", "label-707");
        assert_eq!(item.status, DashboardSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_dashboard_slot("code-483", "label-707");
        if item.move_to(DashboardSlotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardSlotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_dashboard_slot("code-483", "label-707");
        item.status = DashboardSlotStatus::Withdrawn;
        assert!(
            item.move_to(DashboardSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
