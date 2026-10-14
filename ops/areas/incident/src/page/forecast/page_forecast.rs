use super::*;

use acme_kit::text;

/// Where a page forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PageForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageForecast {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub quantity: i64,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub port: i64,
    pub external_ref: String,
    pub status: PageForecastStatus,
}

impl PageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageForecastStatus) -> bool {
        match self.status {
            PageForecastStatus::Proposed => {
                match next {
                    PageForecastStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageForecastStatus::Approved => {
                return next == PageForecastStatus::Withdrawn;
            }
            PageForecastStatus::Rejected => {
                match next {
                    PageForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page forecast {} cannot move from {} to {}",
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
        format!("{} {}", "page forecast", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageForecast {
        PageForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            port: self.port,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a page forecast in status proposed.
pub fn new_page_forecast(code: &str, label: &str) -> PageForecast {
    PageForecast {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        quantity: 0,
        duration_minutes: 0,
        expires_at: 0,
        port: 0,
        external_ref: "".to_string(),
        status: PageForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_page_forecast("code-13", "label-966");
        assert_eq!(item.status, PageForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_page_forecast("code-13", "label-966");
        if item.move_to(PageForecastStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageForecastStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_page_forecast("code-13", "label-966");
        item.status = PageForecastStatus::Withdrawn;
        assert!(
            item.move_to(PageForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
