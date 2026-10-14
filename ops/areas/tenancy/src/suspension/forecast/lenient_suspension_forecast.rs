use super::*;

use acme_kit::text;

/// Where a lenient suspension forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSuspensionForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientSuspensionForecastStatus {
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

impl std::fmt::Display for LenientSuspensionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient suspension forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionForecast {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub active: bool,
    pub phone: String,
    pub threshold: f64,
    pub amount: f64,
    pub source: String,
    pub category: String,
    pub expires_at: i64,
    pub status: LenientSuspensionForecastStatus,
}

impl LenientSuspensionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSuspensionForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSuspensionForecastStatus) -> bool {
        match self.status {
            LenientSuspensionForecastStatus::Draft => {
                return next == LenientSuspensionForecastStatus::Pending;
            }
            LenientSuspensionForecastStatus::Pending => {
                match next {
                    LenientSuspensionForecastStatus::Active | LenientSuspensionForecastStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionForecastStatus::Active => {
                match next {
                    LenientSuspensionForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSuspensionForecastStatus::Suspended => {
                match next {
                    LenientSuspensionForecastStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientSuspensionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient suspension forecast {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSuspensionForecast {
        LenientSuspensionForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            active: self.active,
            phone: self.phone.clone(),
            threshold: self.threshold,
            amount: self.amount,
            source: self.source.clone(),
            category: self.category.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient suspension forecast in status draft.
pub fn new_lenient_suspension_forecast(code: &str, label: &str) -> LenientSuspensionForecast {
    LenientSuspensionForecast {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        active: false,
        phone: "".to_string(),
        threshold: 0.0,
        amount: 0.0,
        source: "".to_string(),
        category: "".to_string(),
        expires_at: 0,
        status: LenientSuspensionForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_suspension_forecast("code-837", "label-615");
        assert_eq!(item.status, LenientSuspensionForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_suspension_forecast("code-837", "label-615");
        if item.move_to(LenientSuspensionForecastStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSuspensionForecastStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_suspension_forecast("code-837", "label-615");
        item.status = LenientSuspensionForecastStatus::Closed;
        assert!(
            item.move_to(LenientSuspensionForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
