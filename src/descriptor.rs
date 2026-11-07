//! Strict, versioned parameter descriptor. It never embeds executable code or paths.
use crate::{AvatarPose, AvatarStyle, Frame, Gaze};
use serde::{Deserialize, Serialize};

pub const MAX_DESCRIPTOR_BYTES: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    #[serde(deserialize_with = "schema_version")]
    pub schema_version: u32,
    pub style: String,
    pub state: String,
    pub time: f32,
    pub level: f32,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub gaze: Option<[f32; 2]>,
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub bands: Option<[f32; 8]>,
}

fn schema_version<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let number = serde_json::Number::deserialize(deserializer)?;
    if number.as_f64() == Some(1.0) {
        Ok(1)
    } else {
        Err(serde::de::Error::custom(
            "Unsupported avatar schema_version",
        ))
    }
}

fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

impl Descriptor {
    pub fn from_json(value: &str) -> Result<Self, String> {
        if value.len() > MAX_DESCRIPTOR_BYTES {
            return Err("Avatar descriptor exceeds 4096 bytes".into());
        }
        let result: Self =
            serde_json::from_str(value).map_err(|e| format!("Invalid avatar descriptor: {e}"))?;
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("Unsupported avatar schema_version".into());
        }
        // Any bounded id is accepted: retired and unknown ids resolve to a drawable
        // design (AvatarStyle::resolve), so a stored identity never fails to render.
        if self.style.is_empty() || self.style.len() > 64 {
            return Err("Avatar style must be a nonempty id of at most 64 bytes".into());
        }
        if AvatarPose::from_id(&self.state).is_none() {
            return Err("Unknown avatar state".into());
        }
        if !self.time.is_finite() || !(0.0..=86400.0).contains(&self.time) {
            return Err("Avatar time must be finite seconds in 0..86400".into());
        }
        if !self.level.is_finite() || !(0.0..=1.0).contains(&self.level) {
            return Err("Avatar level must be finite in 0..1".into());
        }
        if self.gaze.is_some_and(|g| {
            g.iter()
                .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        }) {
            return Err("Avatar gaze must contain two finite values in -1..1".into());
        }
        if self
            .bands
            .is_some_and(|b| b.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)))
        {
            return Err("Avatar bands must contain eight finite values in 0..1".into());
        }
        Ok(())
    }
    pub fn frame(&self) -> Result<Frame, String> {
        self.validate()?;
        let gaze = self.gaze.unwrap_or([0., 0.]);
        Ok(Frame::build(
            AvatarStyle::resolve(&self.style),
            AvatarPose::from_id(&self.state).unwrap(),
            self.level,
            self.time,
            Gaze::new(gaze[0], gaze[1]),
            self.bands,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_bounded_requests_and_rejects_code_unknown_fields_and_nan() {
        let good = r#"{"schema_version":1,"style":"halo","state":"ready","time":1.2,"level":0.5}"#;
        assert!(Descriptor::from_json(good).unwrap().frame().is_ok());
        let serialized = serde_json::to_string(&Descriptor::from_json(good).unwrap()).unwrap();
        assert!(!serialized.contains("null"));
        assert!(Descriptor::from_json(&serialized).is_ok());
        assert!(Descriptor::from_json(
            &good.replace("\"schema_version\":1", "\"schema_version\":1.0")
        )
        .is_ok());
        for field in ["gaze", "bands"] {
            let bad = good.replace("\"level\":0.5", &format!("\"level\":0.5,\"{field}\":null"));
            assert!(Descriptor::from_json(&bad).is_err());
        }
        for bad in [
            good.replace("\"schema_version\":1", "\"schema_version\":2"),
            good.replace("\"time\":1.2", "\"time\":-1"),
            good.replace("\"level\":0.5", "\"level\":NaN"),
            good.replace("\"style\":\"halo\"", "\"style\":\"\""),
            good.replace(
                "\"style\":\"halo\"",
                &format!("\"style\":\"{}\"", "x".repeat(65)),
            ),
            good.replace("\"state\":\"ready\"", "\"state\":\"running\""),
            good.replace("\"level\":0.5", "\"level\":0.5,\"script\":\"alert(1)\""),
            good.replace("\"level\":0.5", "\"level\":0.5,\"bands\":[0.1]"),
        ] {
            assert!(Descriptor::from_json(&bad).is_err(), "{bad}");
        }
        assert!(Descriptor::from_json(&" ".repeat(MAX_DESCRIPTOR_BYTES + 1)).is_err());
    }
    #[test]
    fn retired_and_unknown_styles_render_their_documented_fallback() {
        let request = |style: &str| {
            format!(
                r#"{{"schema_version":1,"style":"{style}","state":"speaking","time":1.2,"level":0.5}}"#
            )
        };
        let pixels = |style: &str| {
            let frame = Descriptor::from_json(&request(style))
                .unwrap()
                .frame()
                .unwrap();
            let palette = [[0xffffffff; 256]; 3];
            (0..32 * 32)
                .map(|i| {
                    let (x, y) = ((i % 32) as f32 / 16. - 1., (i / 32) as f32 / 16. - 1.);
                    frame.rgba(x, y, 1. / 16., &palette)
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(pixels("future"), pixels("watch-halo"));
        assert_eq!(pixels("watch-face"), pixels("watch-halo"));
        assert_eq!(pixels("watch-lens"), pixels("watch-hover"));
        assert_eq!(pixels("eye"), pixels("watch-well"));
        assert!(
            pixels("watch-face").iter().any(|p| p[3] > 0),
            "fallback is never blank"
        );
    }
}
