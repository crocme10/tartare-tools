use minidom::Element;
use minidom_ext::OnlyChildElementExt;
use transit_model::objects::Availability;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Accessibility {
    pub wheelchair: Availability,
    pub visual_announcement: Availability,
    pub audible_announcement: Availability,
}

pub fn accessibility(el: &Element) -> Option<Accessibility> {
    fn availability(val: &str) -> Availability {
        match val {
            "true" => Availability::Available,
            "false" => Availability::NotAvailable,
            _ => Availability::InformationNotAvailable,
        }
    }

    let mobility_impaired_access = el.only_child("MobilityImpairedAccess")?.text();
    let limitation = el
        .only_child("limitations")?
        .only_child("AccessibilityLimitation")?;
    let visual_signs_available = limitation.only_child("VisualSignsAvailable")?.text();
    let audio_signs_available = limitation.only_child("AudibleSignalsAvailable")?.text();

    Some(Accessibility {
        wheelchair: availability(&mobility_impaired_access),
        visual_announcement: availability(&visual_signs_available),
        audible_announcement: availability(&audio_signs_available),
    })
}
