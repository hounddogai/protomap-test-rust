use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional usage template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalUsageTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalUsageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional usage template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTemplate {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub host: String,
    pub expires_at: i64,
    pub ratio: f64,
    pub contact_email: String,
    pub rate: f64,
    pub attempts: i64,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub status: RegionalUsageTemplateStatus,
}

impl RegionalUsageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageTemplateStatus) -> bool {
        match self.status {
            RegionalUsageTemplateStatus::Requested => {
                match next {
                    RegionalUsageTemplateStatus::Granted | RegionalUsageTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageTemplateStatus::Granted => {
                match next {
                    RegionalUsageTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageTemplateStatus::Revoked => {
                return next == RegionalUsageTemplateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalUsageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage template {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.expires_at as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalUsageTemplate {
        RegionalUsageTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            host: self.host.clone(),
            expires_at: self.expires_at,
            ratio: self.ratio,
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            attempts: self.attempts,
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional usage template in status requested.
pub fn new_regional_usage_template(code: &str, title: &str) -> RegionalUsageTemplate {
    RegionalUsageTemplate {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        host: "".to_string(),
        expires_at: 0,
        ratio: 0.0,
        contact_email: "".to_string(),
        rate: 0.0,
        attempts: 0,
        attributes: BTreeMap::new(),
        window_days: 0,
        status: RegionalUsageTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_usage_template("code-72", "title-253");
        assert_eq!(item.status, RegionalUsageTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_usage_template("code-72", "title-253");
        if item.move_to(RegionalUsageTemplateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageTemplateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_usage_template("code-72", "title-253");
        item.status = RegionalUsageTemplateStatus::Expired;
        assert!(
            item.move_to(RegionalUsageTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
