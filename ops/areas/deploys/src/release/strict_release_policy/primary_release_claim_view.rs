use super::*;

/// How screens and exports show a primary release claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseClaimView {
    pub code_text: String,
    pub title: String,
    pub burst: String,
    pub unit: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_primary_release_claim_view(item: &PrimaryReleaseClaim) -> PrimaryReleaseClaimView {
    PrimaryReleaseClaimView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        burst: item.burst.to_string(),
        unit: item.unit.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_release_claim_views(items: &[PrimaryReleaseClaim]) -> Vec<PrimaryReleaseClaimView> {
    let mut views: Vec<PrimaryReleaseClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_release_claim_view(item));
    }
    views
}

pub fn sum_primary_release_claim_view(view: &PrimaryReleaseClaimView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_burst as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_release_claim("code-867", "title-398");
        let view = to_primary_release_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_release_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
