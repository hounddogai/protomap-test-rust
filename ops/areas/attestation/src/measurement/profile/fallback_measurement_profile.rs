use super::*;

use acme_kit::text;

/// Where a fallback measurement profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackMeasurementProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackMeasurementProfileStatus {
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

impl std::fmt::Display for FallbackMeasurementProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback measurement profile as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementProfile {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub name: String,
    pub audience: String,
    pub amount: f64,
    pub status: FallbackMeasurementProfileStatus,
}

impl FallbackMeasurementProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackMeasurementProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackMeasurementProfileStatus) -> bool {
        match self.status {
            FallbackMeasurementProfileStatus::Draft => {
                match next {
                    FallbackMeasurementProfileStatus::Pending | FallbackMeasurementProfileStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMeasurementProfileStatus::Pending => {
                match next {
                    FallbackMeasurementProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMeasurementProfileStatus::Active => {
                match next {
                    FallbackMeasurementProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMeasurementProfileStatus::Suspended => {
                return next == FallbackMeasurementProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackMeasurementProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback measurement profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> FallbackMeasurementProfile {
        FallbackMeasurementProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback measurement profile in status draft.
pub fn new_fallback_measurement_profile(code: &str, title: &str) -> FallbackMeasurementProfile {
    FallbackMeasurementProfile {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        name: "".to_string(),
        audience: "".to_string(),
        amount: 0.0,
        status: FallbackMeasurementProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_measurement_profile("code-37", "title-93");
        assert_eq!(item.status, FallbackMeasurementProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_fallback_measurement_profile("code-37", "title-93");
        if item.move_to(FallbackMeasurementProfileStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackMeasurementProfileStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_measurement_profile("code-37", "title-93");
        item.status = FallbackMeasurementProfileStatus::Closed;
        assert!(
            item.move_to(FallbackMeasurementProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
