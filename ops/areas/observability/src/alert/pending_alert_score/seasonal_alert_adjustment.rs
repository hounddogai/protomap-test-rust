use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal alert adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAlertAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalAlertAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SeasonalAlertAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal alert adjustment as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustment {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub ttl_seconds: i64,
    pub host: String,
    pub capacity: i64,
    pub revision: i64,
    pub port: i64,
    pub contact_email: String,
    pub scope: String,
    pub status: SeasonalAlertAdjustmentStatus,
}

impl SeasonalAlertAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAlertAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAlertAdjustmentStatus) -> bool {
        match self.status {
            SeasonalAlertAdjustmentStatus::New => {
                match next {
                    SeasonalAlertAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAlertAdjustmentStatus::Queued => {
                return next == SeasonalAlertAdjustmentStatus::Running || next == SeasonalAlertAdjustmentStatus::Failed;
            }
            SeasonalAlertAdjustmentStatus::Running => {
                return next == SeasonalAlertAdjustmentStatus::Done;
            }
            SeasonalAlertAdjustmentStatus::Done => {
                match next {
                    SeasonalAlertAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalAlertAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal alert adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAlertAdjustment {
        SeasonalAlertAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            ttl_seconds: self.ttl_seconds,
            host: self.host.clone(),
            capacity: self.capacity,
            revision: self.revision,
            port: self.port,
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal alert adjustment in status new.
pub fn new_seasonal_alert_adjustment(code: &str, title: &str) -> SeasonalAlertAdjustment {
    SeasonalAlertAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        ttl_seconds: 0,
        host: "".to_string(),
        capacity: 0,
        revision: 0,
        port: 0,
        contact_email: "".to_string(),
        scope: "".to_string(),
        status: SeasonalAlertAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_alert_adjustment("code-586", "title-26");
        assert_eq!(item.status, SeasonalAlertAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_seasonal_alert_adjustment("code-586", "title-26");
        if item.move_to(SeasonalAlertAdjustmentStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAlertAdjustmentStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_alert_adjustment("code-586", "title-26");
        item.status = SeasonalAlertAdjustmentStatus::Failed;
        assert!(
            item.move_to(SeasonalAlertAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
