use super::*;

use acme_kit::text;

/// Where a egress forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressForecastStatus {
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

impl std::fmt::Display for EgressForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressForecast {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub created_at: i64,
    pub cipher: String,
    pub visible: bool,
    pub scope: String,
    pub status: EgressForecastStatus,
}

impl EgressForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressForecastStatus) -> bool {
        match self.status {
            EgressForecastStatus::Draft => {
                return next == EgressForecastStatus::Pending;
            }
            EgressForecastStatus::Pending => {
                return next == EgressForecastStatus::Closed;
            }
            EgressForecastStatus::Active => {
                return next == EgressForecastStatus::Suspended || next == EgressForecastStatus::Closed;
            }
            EgressForecastStatus::Suspended => {
                return next == EgressForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress forecast", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> EgressForecast {
        EgressForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            visible: self.visible,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a egress forecast in status draft.
pub fn new_egress_forecast(code: &str, label: &str) -> EgressForecast {
    EgressForecast {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        created_at: 0,
        cipher: "".to_string(),
        visible: false,
        scope: "".to_string(),
        status: EgressForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_forecast("code-766", "label-180");
        assert_eq!(item.status, EgressForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_forecast("code-766", "label-180");
        if item.move_to(EgressForecastStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressForecastStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_forecast("code-766", "label-180");
        item.status = EgressForecastStatus::Closed;
        assert!(
            item.move_to(EgressForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
