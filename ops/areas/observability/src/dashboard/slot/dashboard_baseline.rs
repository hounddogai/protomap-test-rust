use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a dashboard baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DashboardBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DashboardBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBaseline {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub namespace: String,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub margin: f64,
    pub updated_at: i64,
    pub summary: String,
    pub region: String,
    pub status: DashboardBaselineStatus,
}

impl DashboardBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardBaselineStatus) -> bool {
        match self.status {
            DashboardBaselineStatus::Planned => {
                match next {
                    DashboardBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardBaselineStatus::Scheduled => {
                match next {
                    DashboardBaselineStatus::InProgress | DashboardBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardBaselineStatus::InProgress => {
                return next == DashboardBaselineStatus::Completed || next == DashboardBaselineStatus::Cancelled;
            }
            DashboardBaselineStatus::Completed => {
                return next == DashboardBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.weight + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardBaseline {
        DashboardBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            namespace: self.namespace.clone(),
            weight: self.weight,
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            updated_at: self.updated_at,
            summary: self.summary.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a dashboard baseline in status planned.
pub fn new_dashboard_baseline(code: &str, title: &str) -> DashboardBaseline {
    DashboardBaseline {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        namespace: "".to_string(),
        weight: 0.0,
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        margin: 0.0,
        updated_at: 0,
        summary: "".to_string(),
        region: "".to_string(),
        status: DashboardBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dashboard_baseline("code-327", "title-841");
        assert_eq!(item.status, DashboardBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_dashboard_baseline("code-327", "title-841");
        if item.move_to(DashboardBaselineStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardBaselineStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dashboard_baseline("code-327", "title-841");
        item.status = DashboardBaselineStatus::Cancelled;
        assert!(
            item.move_to(DashboardBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
