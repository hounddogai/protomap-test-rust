use super::*;

use acme_kit::text;

/// Where a alert score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertScore {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub title: String,
    pub starts_at: i64,
    pub budget: f64,
    pub window_days: i64,
    pub status: AlertScoreStatus,
}

impl AlertScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertScoreStatus) -> bool {
        match self.status {
            AlertScoreStatus::Created => {
                match next {
                    AlertScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertScoreStatus::Validated => {
                match next {
                    AlertScoreStatus::Published | AlertScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertScoreStatus::Published => {
                match next {
                    AlertScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: AlertScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert score {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.floor_value - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertScore {
        AlertScore {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            title: self.title.clone(),
            starts_at: self.starts_at,
            budget: self.budget,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a alert score in status created.
pub fn new_alert_score(code: &str, name: &str) -> AlertScore {
    AlertScore {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        title: "".to_string(),
        starts_at: 0,
        budget: 0.0,
        window_days: 0,
        status: AlertScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_alert_score("code-923", "name-428");
        assert_eq!(item.status, AlertScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_alert_score("code-923", "name-428");
        if item.move_to(AlertScoreStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertScoreStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_alert_score("code-923", "name-428");
        item.status = AlertScoreStatus::Retired;
        assert!(
            item.move_to(AlertScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
