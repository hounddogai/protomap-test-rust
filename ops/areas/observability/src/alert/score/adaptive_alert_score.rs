use super::*;

use acme_kit::text;

/// Where a adaptive alert score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAlertScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveAlertScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveAlertScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive alert score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertScore {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub label: String,
    pub active: bool,
    pub channel: String,
    pub cluster: String,
    pub amount: f64,
    pub currency: String,
    pub status: AdaptiveAlertScoreStatus,
}

impl AdaptiveAlertScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAlertScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAlertScoreStatus) -> bool {
        match self.status {
            AdaptiveAlertScoreStatus::Created => {
                return next == AdaptiveAlertScoreStatus::Published;
            }
            AdaptiveAlertScoreStatus::Validated => {
                match next {
                    AdaptiveAlertScoreStatus::Published | AdaptiveAlertScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAlertScoreStatus::Published => {
                match next {
                    AdaptiveAlertScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: AdaptiveAlertScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive alert score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.amount * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAlertScore {
        AdaptiveAlertScore {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            label: self.label.clone(),
            active: self.active,
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            amount: self.amount,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive alert score in status created.
pub fn new_adaptive_alert_score(code: &str, title: &str) -> AdaptiveAlertScore {
    AdaptiveAlertScore {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        duration_minutes: 0,
        label: "".to_string(),
        active: false,
        channel: "".to_string(),
        cluster: "".to_string(),
        amount: 0.0,
        currency: "".to_string(),
        status: AdaptiveAlertScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_alert_score("code-409", "title-951");
        assert_eq!(item.status, AdaptiveAlertScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_adaptive_alert_score("code-409", "title-951");
        if item.move_to(AdaptiveAlertScoreStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAlertScoreStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_alert_score("code-409", "title-951");
        item.status = AdaptiveAlertScoreStatus::Retired;
        assert!(
            item.move_to(AdaptiveAlertScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
