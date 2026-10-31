use num_rational::Ratio;

fn ratio_f64(value: &Ratio<i64>) -> f64 {
    *value.numer() as f64 / *value.denom() as f64
}

pub const TARGET_WIDTH: u32 = 1920;
pub const TARGET_HEIGHT: u32 = 1080;

pub fn validate_target(width: u32, height: u32) -> Result<(), String> {
    if width < 16 || height < 16 {
        return Err("Width and height must be at least 16 pixels.".to_string());
    }
    if width > 7680 || height > 7680 {
        return Err("Width and height cannot exceed 7680 pixels.".to_string());
    }
    if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
        return Err("Width and height must be even numbers.".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasPlan {
    pub filter_chain: Option<String>,
    pub scaled_width: u32,
    pub scaled_height: u32,
    pub pad_x: u32,
    pub pad_y: u32,
    pub target_width: u32,
    pub target_height: u32,
    pub scale_needed: bool,
    pub notes: Vec<String>,
}

impl CanvasPlan {
    pub fn pad_text(&self) -> String {
        format!("{} px left/right, {} px top/bottom", self.pad_x, self.pad_y)
    }
}

fn parse_ratio(value: &str) -> Option<Ratio<i64>> {
    let text = value.trim();
    if text.is_empty() || matches!(text, "0:1" | "N/A" | "0/0") {
        return None;
    }
    let (left, right) = if let Some((a, b)) = text.split_once(':') {
        (a, b)
    } else if let Some((a, b)) = text.split_once('/') {
        (a, b)
    } else {
        return None;
    };
    let numerator: i64 = left.parse().ok()?;
    let denominator: i64 = right.parse().ok()?;
    if numerator <= 0 || denominator <= 0 {
        return None;
    }
    Some(Ratio::new(numerator, denominator))
}

fn even(value: f64, maximum: u32) -> u32 {
    let mut rounded = ((value / 2.0).round() as u32) * 2;
    if rounded < 2 {
        rounded = 2;
    }
    if rounded > maximum {
        rounded = maximum - (maximum % 2);
    }
    rounded
}

fn normalize_rotation(rotation: i32) -> i32 {
    rotation.rem_euclid(360)
}

pub fn compute_canvas(
    width: u32,
    height: u32,
    sample_aspect_ratio: &str,
    display_aspect_ratio: &str,
    rotation: i32,
    target_width: u32,
    target_height: u32,
) -> CanvasPlan {
    assert!(width > 0 && height > 0, "video dimensions must be positive");
    assert!(
        target_width > 0 && target_height > 0,
        "canvas dimensions must be positive"
    );

    let rotation = normalize_rotation(rotation);
    let (effective_width, effective_height) = if rotation == 90 || rotation == 270 {
        (height, width)
    } else {
        (width, height)
    };

    let sar = parse_ratio(sample_aspect_ratio);
    let dar = parse_ratio(display_aspect_ratio);
    let display_ar = if let Some(sar) = sar {
        sar * Ratio::new(effective_width as i64, effective_height as i64)
    } else if let Some(dar) = dar {
        dar
    } else {
        Ratio::new(effective_width as i64, effective_height as i64)
    };
    let target_ar = Ratio::new(target_width as i64, target_height as i64);

    let (scaled_width, scaled_height) = if display_ar >= target_ar {
        (
            target_width,
            even(target_width as f64 / ratio_f64(&display_ar), target_height),
        )
    } else {
        (
            even(target_height as f64 * ratio_f64(&display_ar), target_width),
            target_height,
        )
    };

    let pad_x = (target_width.saturating_sub(scaled_width)) / 2;
    let pad_y = (target_height.saturating_sub(scaled_height)) / 2;
    let scale_needed = (scaled_width, scaled_height) != (effective_width, effective_height);

    let mut notes = Vec::new();
    if rotation != 0 {
        notes.push(format!("rotation {rotation}°"));
    }
    if let Some(sar) = sar
        && sar != Ratio::new(1, 1)
    {
        notes.push(format!("SAR {}/{}", sar.numer(), sar.denom()));
    }
    if width == target_width
        && height == target_height
        && rotation == 0
        && sar.is_none_or(|value| value == Ratio::new(1, 1))
    {
        notes.push("already 1920×1080".to_string());
    }

    let mut filters: Vec<String> = Vec::new();
    if scale_needed {
        filters.push(format!(
            "scale={scaled_width}:{scaled_height}:flags=lanczos"
        ));
    }
    if scale_needed || sar.is_some_and(|value| value != Ratio::new(1, 1)) {
        filters.push("setsar=1".to_string());
    }
    if scaled_width != target_width || scaled_height != target_height {
        filters.push(format!(
            "pad={target_width}:{target_height}:{pad_x}:{pad_y}:black"
        ));
    }

    CanvasPlan {
        filter_chain: if filters.is_empty() {
            None
        } else {
            Some(filters.join(","))
        },
        scaled_width,
        scaled_height,
        pad_x,
        pad_y,
        target_width,
        target_height,
        scale_needed,
        notes,
    }
}

pub fn canvas_for_stream(
    stream: &crate::models::StreamInfo,
    target_width: u32,
    target_height: u32,
) -> CanvasPlan {
    compute_canvas(
        stream.width.unwrap_or(0),
        stream.height.unwrap_or(0),
        &stream.sample_aspect_ratio,
        &stream.display_aspect_ratio,
        stream.rotation,
        target_width,
        target_height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_hd_needs_no_filter() {
        let plan = compute_canvas(1920, 1080, "1:1", "16:9", 0, 1920, 1080);
        assert_eq!(plan.filter_chain, None);
        assert_eq!((plan.pad_x, plan.pad_y), (0, 0));
    }

    #[test]
    fn widely_used_padding_values() {
        assert_eq!(
            compute_canvas(1920, 804, "1:1", "160:67", 0, 1920, 1080)
                .filter_chain
                .as_deref(),
            Some("pad=1920:1080:0:138:black")
        );
        assert_eq!(
            compute_canvas(1920, 800, "1:1", "12:5", 0, 1920, 1080)
                .filter_chain
                .as_deref(),
            Some("pad=1920:1080:0:140:black")
        );
        assert_eq!(
            compute_canvas(1920, 1040, "1:1", "", 0, 1920, 1080)
                .filter_chain
                .as_deref(),
            Some("pad=1920:1080:0:20:black")
        );
    }

    #[test]
    fn scales_and_pads() {
        let plan = compute_canvas(1280, 720, "1:1", "", 0, 1920, 1080);
        assert_eq!(
            plan.filter_chain.as_deref(),
            Some("scale=1920:1080:flags=lanczos,setsar=1")
        );
        let plan = compute_canvas(1440, 1080, "1:1", "", 0, 1920, 1080);
        assert_eq!(
            plan.filter_chain.as_deref(),
            Some("pad=1920:1080:240:0:black")
        );
    }

    #[test]
    fn anamorphic_sar_is_respected() {
        let plan = compute_canvas(1440, 1080, "4:3", "", 0, 1920, 1080);
        assert_eq!(
            plan.filter_chain.as_deref(),
            Some("scale=1920:1080:flags=lanczos,setsar=1")
        );
    }

    #[test]
    fn rotation_swaps_dimensions() {
        let plan = compute_canvas(1080, 1920, "1:1", "", 90, 1920, 1080);
        assert_eq!(plan.filter_chain, None);
        assert!(plan.notes.iter().any(|note| note.contains("90")));
    }

    #[test]
    fn odd_dimensions_become_even() {
        let plan = compute_canvas(1919, 803, "1:1", "", 0, 1920, 1080);
        assert_eq!(plan.scaled_width % 2, 0);
        assert_eq!(plan.scaled_height % 2, 0);
    }

    #[test]
    fn narrow_input_pillarboxes() {
        let plan = compute_canvas(100, 100, "1:1", "", 0, 1920, 1080);
        assert_eq!((plan.scaled_width, plan.scaled_height), (1080, 1080));
        assert_eq!(plan.pad_x, 420);
    }

    #[test]
    fn validates_target_dimensions() {
        assert!(validate_target(1920, 1080).is_ok());
        assert!(validate_target(1280, 720).is_ok());
        assert!(validate_target(1919, 1080).is_err());
        assert!(validate_target(0, 1080).is_err());
        assert!(validate_target(1920, 9000).is_err());
    }

    #[test]
    fn custom_canvas_scales_and_pads() {
        let plan = compute_canvas(1920, 804, "1:1", "", 0, 1600, 900);
        assert_eq!((plan.target_width, plan.target_height), (1600, 900));
        assert!(plan.pad_y > 0);
    }
}
