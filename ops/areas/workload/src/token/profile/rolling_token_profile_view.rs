use super::*;

/// How screens and exports show a rolling token profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenProfileView {
    pub code: String,
    pub label_text: String,
    pub score: String,
    pub grace_days: String,
    pub issuer: String,
    pub checksum_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_rolling_token_profile_view(item: &RollingTokenProfile) -> RollingTokenProfileView {
    RollingTokenProfileView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        score: format!("{:.2}", item.score),
        grace_days: item.grace_days.to_string(),
        issuer: item.issuer.clone(),
        checksum_text: item.checksum.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_token_profile_views(items: &[RollingTokenProfile]) -> Vec<RollingTokenProfileView> {
    let mut views: Vec<RollingTokenProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_token_profile_view(item));
    }
    views
}

pub fn sum_rolling_token_profile_view(view: &RollingTokenProfileView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_score + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_token_profile("code-482", "label-560");
        let view = to_rolling_token_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_token_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
