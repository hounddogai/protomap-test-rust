use super::*;

use acme_kit::text;

/// Where a regional alert claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalAlertClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalAlertClaimStatus {
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

impl std::fmt::Display for RegionalAlertClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional alert claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAlertClaim {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub fingerprint: String,
    pub source: String,
    pub rotation_days: i64,
    pub status: RegionalAlertClaimStatus,
}

impl RegionalAlertClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalAlertClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalAlertClaimStatus) -> bool {
        match self.status {
            RegionalAlertClaimStatus::Draft => {
                return next == RegionalAlertClaimStatus::Closed;
            }
            RegionalAlertClaimStatus::Pending => {
                match next {
                    RegionalAlertClaimStatus::Active | RegionalAlertClaimStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalAlertClaimStatus::Active => {
                return next == RegionalAlertClaimStatus::Suspended;
            }
            RegionalAlertClaimStatus::Suspended => {
                match next {
                    RegionalAlertClaimStatus::Closed => {
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

    pub fn move_to(&mut self, next: RegionalAlertClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional alert claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.maximum * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalAlertClaim {
        RegionalAlertClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a regional alert claim in status draft.
pub fn new_regional_alert_claim(code: &str, title: &str) -> RegionalAlertClaim {
    RegionalAlertClaim {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        fingerprint: "".to_string(),
        source: "".to_string(),
        rotation_days: 0,
        status: RegionalAlertClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_alert_claim("code-623", "title-901");
        assert_eq!(item.status, RegionalAlertClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_regional_alert_claim("code-623", "title-901");
        if item.move_to(RegionalAlertClaimStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalAlertClaimStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_alert_claim("code-623", "title-901");
        item.status = RegionalAlertClaimStatus::Closed;
        assert!(
            item.move_to(RegionalAlertClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
