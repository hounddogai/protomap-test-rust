use super::*;

use acme_kit::text;

/// Where a lenient silence tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSilenceTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientSilenceTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientSilenceTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient silence tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTier {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub amount: f64,
    pub weight: f64,
    pub capacity: i64,
    pub issuer: String,
    pub maximum: f64,
    pub rotation_days: i64,
    pub retries: i64,
    pub status: LenientSilenceTierStatus,
}

impl LenientSilenceTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSilenceTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSilenceTierStatus) -> bool {
        match self.status {
            LenientSilenceTierStatus::Created => {
                return next == LenientSilenceTierStatus::Validated || next == LenientSilenceTierStatus::Published;
            }
            LenientSilenceTierStatus::Validated => {
                match next {
                    LenientSilenceTierStatus::Published | LenientSilenceTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSilenceTierStatus::Published => {
                match next {
                    LenientSilenceTierStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientSilenceTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient silence tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.amount + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSilenceTier {
        LenientSilenceTier {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            amount: self.amount,
            weight: self.weight,
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient silence tier in status created.
pub fn new_lenient_silence_tier(code: &str, label: &str) -> LenientSilenceTier {
    LenientSilenceTier {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        amount: 0.0,
        weight: 0.0,
        capacity: 0,
        issuer: "".to_string(),
        maximum: 0.0,
        rotation_days: 0,
        retries: 0,
        status: LenientSilenceTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_silence_tier("code-966", "label-610");
        assert_eq!(item.status, LenientSilenceTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_silence_tier("code-966", "label-610");
        if item.move_to(LenientSilenceTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSilenceTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_silence_tier("code-966", "label-610");
        item.status = LenientSilenceTierStatus::Retired;
        assert!(
            item.move_to(LenientSilenceTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
