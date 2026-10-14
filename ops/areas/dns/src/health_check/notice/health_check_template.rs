use super::*;

use acme_kit::text;

/// Where a health check template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl HealthCheckTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for HealthCheckTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTemplate {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub channel: String,
    pub score: f64,
    pub quota_units: i64,
    pub status: HealthCheckTemplateStatus,
}

impl HealthCheckTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckTemplateStatus) -> bool {
        match self.status {
            HealthCheckTemplateStatus::Requested => {
                match next {
                    HealthCheckTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckTemplateStatus::Granted => {
                return next == HealthCheckTemplateStatus::Revoked || next == HealthCheckTemplateStatus::Expired;
            }
            HealthCheckTemplateStatus::Revoked => {
                match next {
                    HealthCheckTemplateStatus::Expired => {
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

    pub fn move_to(&mut self, next: HealthCheckTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check template {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.quota_units as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckTemplate {
        HealthCheckTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            score: self.score,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a health check template in status requested.
pub fn new_health_check_template(code: &str, label: &str) -> HealthCheckTemplate {
    HealthCheckTemplate {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        channel: "".to_string(),
        score: 0.0,
        quota_units: 0,
        status: HealthCheckTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_health_check_template("code-282", "label-227");
        assert_eq!(item.status, HealthCheckTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_health_check_template("code-282", "label-227");
        if item.move_to(HealthCheckTemplateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckTemplateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_health_check_template("code-282", "label-227");
        item.status = HealthCheckTemplateStatus::Expired;
        assert!(
            item.move_to(HealthCheckTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
