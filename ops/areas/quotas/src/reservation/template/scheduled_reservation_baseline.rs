use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled reservation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledReservationBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledReservationBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledReservationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled reservation baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationBaseline {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub audience: String,
    pub created_at: i64,
    pub visible: bool,
    pub cluster: String,
    pub source: String,
    pub duration_minutes: i64,
    pub status: ScheduledReservationBaselineStatus,
}

impl ScheduledReservationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledReservationBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledReservationBaselineStatus) -> bool {
        match self.status {
            ScheduledReservationBaselineStatus::Created => {
                return next == ScheduledReservationBaselineStatus::Published;
            }
            ScheduledReservationBaselineStatus::Validated => {
                match next {
                    ScheduledReservationBaselineStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReservationBaselineStatus::Published => {
                return next == ScheduledReservationBaselineStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledReservationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled reservation baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn projected_total(&self) -> f64 {
        ((self.expires_at as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledReservationBaseline {
        ScheduledReservationBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            audience: self.audience.clone(),
            created_at: self.created_at,
            visible: self.visible,
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled reservation baseline in status created.
pub fn new_scheduled_reservation_baseline(code: &str, title: &str) -> ScheduledReservationBaseline {
    ScheduledReservationBaseline {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        name: "".to_string(),
        attributes: BTreeMap::new(),
        audience: "".to_string(),
        created_at: 0,
        visible: false,
        cluster: "".to_string(),
        source: "".to_string(),
        duration_minutes: 0,
        status: ScheduledReservationBaselineStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_reservation_baseline("code-216", "title-155");
        assert_eq!(item.status, ScheduledReservationBaselineStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_scheduled_reservation_baseline("code-216", "title-155");
        if item.move_to(ScheduledReservationBaselineStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledReservationBaselineStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_reservation_baseline("code-216", "title-155");
        item.status = ScheduledReservationBaselineStatus::Retired;
        assert!(
            item.move_to(ScheduledReservationBaselineStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
