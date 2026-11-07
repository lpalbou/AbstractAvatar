//! Procedural design catalog. Persist string ids: a numeric ABI code is only a position in
//! this build's catalog and changes when a design is retired. A retired or unknown id
//! resolves to a documented design through [`AvatarStyle::resolve`], so a stored identity
//! always renders.
//!
//! Every active design carries a [`DesignStatus`]. Only [`DesignStatus::Ready`] designs are
//! offered to users ([`AvatarStyle::ready`]); [`DesignStatus::Experimental`] designs still
//! render when a host asks for them by id, but pickers and defaults never surface them.

/// Whether an active design may be offered to users.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DesignStatus {
    /// Finished: hosts may list it in pickers and use it as a default.
    Ready,
    /// Not ready: it renders when asked for by id, but hosts must not offer it.
    Experimental,
}
impl DesignStatus {
    /// The catalog value: `"ready"` or `"experimental"`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Experimental => "experimental",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum AvatarStyle {
    Halo,
    Hover,
    Vortex,
    Eclipse,
    Rover,
    #[default]
    WatchHalo,
    WatchVortex,
    WatchNebula,
    WatchRings,
    WatchLiquid,
    WatchGalaxy,
    WatchLotus,
    WatchCrystal,
    WatchEclipse,
    WatchSynapse,
    WatchWave,
    WatchWell,
    WatchBlackHole,
    WatchAccretion,
    WatchAurora,
    WatchMurmuration,
    WatchEye,
    WatchRover,
    WatchHover,
    WatchSpark,
    WatchOcean,
    IphoneWeave,
}
impl AvatarStyle {
    pub const ALL: [Self; 27] = [
        Self::Halo,
        Self::Hover,
        Self::Vortex,
        Self::Eclipse,
        Self::Rover,
        Self::WatchHalo,
        Self::WatchVortex,
        Self::WatchNebula,
        Self::WatchRings,
        Self::WatchLiquid,
        Self::WatchGalaxy,
        Self::WatchLotus,
        Self::WatchCrystal,
        Self::WatchEclipse,
        Self::WatchSynapse,
        Self::WatchWave,
        Self::WatchWell,
        Self::WatchBlackHole,
        Self::WatchAccretion,
        Self::WatchAurora,
        Self::WatchMurmuration,
        Self::WatchEye,
        Self::WatchRover,
        Self::WatchHover,
        Self::WatchSpark,
        Self::WatchOcean,
        Self::IphoneWeave,
    ];
    /// Every active design in catalog order, ready and experimental. Use [`Self::ready`]
    /// for anything a user chooses from.
    pub fn all() -> impl Iterator<Item = Self> {
        Self::ALL.into_iter()
    }
    /// The designs a host may offer (pickers, lists, defaults), in catalog order.
    pub fn ready() -> impl Iterator<Item = Self> {
        Self::all().filter(|s| s.is_ready())
    }
    /// Whether this design is offered to users or only reachable by id.
    pub fn status(self) -> DesignStatus {
        match self {
            Self::IphoneWeave
            | Self::WatchHover
            | Self::WatchMurmuration
            | Self::WatchWell
            | Self::WatchWave
            | Self::WatchRings
            | Self::WatchHalo => DesignStatus::Ready,
            _ => DesignStatus::Experimental,
        }
    }
    pub fn is_ready(self) -> bool {
        self.status() == DesignStatus::Ready
    }
    pub fn from_code(code: u32) -> Option<Self> {
        Self::ALL.get(code as usize).copied()
    }
    pub fn code(self) -> u32 {
        Self::ALL.iter().position(|v| *v == self).unwrap() as u32
    }
    /// The design an unknown id resolves to, and the replacement for retired designs that
    /// name none: the ready [`Self::WatchHalo`] orb.
    pub const DEFAULT: Self = Self::WatchHalo;
    /// Look up an active design, ready or experimental. Retired and unknown ids return
    /// `None`; use [`Self::resolve`] to render a stored identity.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }
    /// The design to draw for a stored id, never failing: an active id is itself, a retired
    /// id is its replacement (or [`Self::DEFAULT`] when it names none), and any other id is
    /// [`Self::DEFAULT`].
    pub fn resolve(id: &str) -> Self {
        Self::from_id(id)
            .or_else(|| Self::retired(id).and_then(|r| r.replacement))
            .unwrap_or(Self::DEFAULT)
    }
    /// The retirement record for an id that is no longer in the active catalog.
    pub fn retired(id: &str) -> Option<&'static RetiredStyle> {
        RETIRED.iter().find(|r| r.id == id)
    }
    /// The five shaded designs sampled analytically; every other design is a stroke scene.
    pub(crate) fn analytic(self) -> bool {
        matches!(
            self,
            Self::Halo | Self::Hover | Self::Vortex | Self::Eclipse | Self::Rover
        )
    }
    pub fn id(self) -> &'static str {
        self.fields()[0]
    }
    pub fn label(self) -> &'static str {
        self.fields()[1]
    }
    pub fn description(self) -> &'static str {
        self.fields()[2]
    }
    pub fn watch_code(self) -> &'static str {
        self.fields()[3]
    }
    pub fn family(self) -> &'static str {
        self.fields()[4]
    }
    /// The one component that may follow a host-supplied gaze: `"eyes"` or `"none"`.
    ///
    /// Only designs that draw distinct eyes (pupils, irises or eye lamps) answer gaze,
    /// and only by shifting those eyes; every other design ignores it entirely.
    pub fn gaze(self) -> &'static str {
        if self.follows_gaze() {
            "eyes"
        } else {
            "none"
        }
    }
    pub(crate) fn follows_gaze(self) -> bool {
        matches!(
            self,
            Self::Hover
                | Self::Rover
                | Self::WatchEye
                | Self::WatchRover
                | Self::WatchHover
                | Self::WatchSpark
        )
    }
    /// Catalog field by ABI index: 0..4 as in [`Self::fields`], 5 = [`Self::gaze`],
    /// 6 = [`Self::status`].
    #[cfg(feature = "ffi")]
    pub(crate) fn field(self, index: u32) -> Option<&'static str> {
        match index {
            5 => Some(self.gaze()),
            6 => Some(self.status().id()),
            _ => self.fields().get(index as usize).copied(),
        }
    }
    pub fn fields(self) -> [&'static str; 5] {
        match self{
        Self::Halo => ["halo", "Halo orb", "A luminous corona", "A", "orbs"],
        Self::Hover => ["hover", "Hover robot", "Warm eyes and floating thrusters", "HV", "machines"],
        Self::Vortex => ["vortex", "Vortex", "Spiral arms gather to listen and reach out to speak", "B", "orbs"],
        Self::Eclipse => ["eclipse", "Eclipse", "A dark disc, golden corona and working rays", "U", "forms"],
        Self::Rover => ["rover", "Rover", "A curious tracked robot with attentive binocular eyes", "Q", "machines"],
        Self::WatchHalo => ["watch-halo", "Watch Halo", "rays of light around it, as long as your voice is loud", "A", "orbs"],
        Self::WatchVortex => ["watch-vortex", "Watch Vortex", "the points swirl out into arms that reach with the voice", "B", "orbs"],
        Self::WatchNebula => ["watch-nebula", "Watch Nebula", "a cloud that breathes outward with the voice", "C", "orbs"],
        Self::WatchRings => ["watch-rings", "Watch Rings", "a geodesic structure of rigid rings, each turning with its own part of the voice", "D", "orbs"],
        Self::WatchLiquid => ["watch-liquid", "Watch Liquid", "its surface ripples in waves with the voice", "E", "orbs"],
        Self::WatchGalaxy => ["watch-galaxy", "Watch Galaxy", "a spiral galaxy that turns and spreads its arms with the voice", "F", "orbs"],
        Self::WatchLotus => ["watch-lotus", "Watch Lotus", "a bud that listens, opens to think, branches out to work, blooms to speak", "R", "forms"],
        Self::WatchCrystal => ["watch-crystal", "Watch Crystal", "a crystal that grows to think, gathers orbits to work, spreads wings to speak", "S", "forms"],
        Self::WatchEclipse => ["watch-eclipse", "Watch Eclipse", "a golden eclipse: a corona to listen, orbits to think, a shutter to work, bars to speak", "U", "forms"],
        Self::WatchSynapse => ["watch-synapse", "Watch Synapse", "a network of nodes: tangled to think, orbiting to work, a starburst to speak", "V", "forms"],
        Self::WatchWave => ["watch-wave", "Watch Wave", "ribbons of light, each a harmonic of the voice; they wind into a swirl to work", "X", "forms"],
        Self::WatchWell => ["watch-well", "Watch Well", "space bending into a well: ripples carry the voice, two masses circle it as it works", "T", "cosmos"],
        Self::WatchBlackHole => ["watch-black-hole", "Watch Black hole", "a ring of light round a shadow: gas falls in to listen, a disk to think, jets to work", "Y", "cosmos"],
        Self::WatchAccretion => ["watch-accretion", "Watch Accretion", "a young star's disk of dust: it gathers into clumps to think and merges into planets to work", "Z", "cosmos"],
        Self::WatchAurora => ["watch-aurora", "Watch Aurora", "the northern lights over a lake: they rise with the voice and gather into a corona to think", "AU", "nature"],
        Self::WatchMurmuration => ["watch-murmuration", "Watch Murmuration", "a flock of starlings at dusk: it swells to listen, spins to think, streams to work, draws the voice", "MU", "nature"],
        Self::WatchEye => ["watch-eye", "Watch Eye", "one friendly eye: its iris glows with its voice, it opens wide to listen", "K", "faces"],
        Self::WatchRover => ["watch-rover", "Watch Rover", "a shy, curious little rover on tracks: its eyes lean in to listen, tilt when puzzled, bob when it talks", "Q", "machines"],
        Self::WatchHover => ["watch-hover", "Watch Hover", "a round floating robot, polite and brave: its red eyes glow with its voice, it drifts closer to listen", "HV", "machines"],
        Self::WatchSpark => ["watch-spark", "Watch Spark", "a curious robot on tracks with shutter eyes: it leans in for input and its eyes flick while it thinks", "SP", "machines"],
        Self::WatchOcean => ["watch-ocean", "Watch Ocean", "a sea at dusk whose swell rises with its voice; while it thinks the waves fall into order", "H", "nature"],
        Self::IphoneWeave => ["iphone-weave", "Woven orb", "Five cords become a geodesic sphere or a raised approval hand", "", "forms"],
    }
    }
}

/// A design removed from the active catalog. Its id stays reserved and resolves to
/// `replacement`, or to [`AvatarStyle::DEFAULT`] when that is `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetiredStyle {
    pub id: &'static str,
    /// The version whose catalog first omits the design.
    pub retired_in: &'static str,
    /// The date the removal was decided (YYYY-MM-DD).
    pub date: &'static str,
    pub reason: &'static str,
    pub replacement: Option<AvatarStyle>,
}
const REMOVED: &str = "removed from the catalog";
const REDESIGN: &str =
    "removed until it is redesigned faithfully from its Apple Watch original; a redesign returns under the same id";
const fn retired(
    id: &'static str,
    reason: &'static str,
    replacement: Option<AvatarStyle>,
) -> RetiredStyle {
    RetiredStyle {
        id,
        retired_in: "0.2.0",
        date: "2026-11-07",
        reason,
        replacement,
    }
}
/// Every retired id, in former catalog order. `catalog.json` carries the same list.
pub const RETIRED: [RetiredStyle; 17] = [
    retired("eye", REMOVED, Some(AvatarStyle::WatchWell)),
    retired("watch-drop", REMOVED, None),
    retired("watch-face", REDESIGN, None),
    retired("watch-imprint", REDESIGN, None),
    retired("watch-contour", REDESIGN, None),
    retired("watch-membrane", REDESIGN, None),
    retired("watch-lens", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-dome", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-valet", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-retro", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-deco", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-cruiser", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-android", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-sentinel", REDESIGN, Some(AvatarStyle::WatchHover)),
    retired("watch-shape-kitty", REMOVED, None),
    retired("watch-shape-face", REMOVED, None),
    retired("watch-shape-wave", REMOVED, Some(AvatarStyle::WatchWave)),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retired_ids_are_not_active_and_resolve_to_a_drawable_design() {
        for r in RETIRED {
            assert_eq!(AvatarStyle::from_id(r.id), None, "{} is still active", r.id);
            let resolved = AvatarStyle::resolve(r.id);
            assert_eq!(resolved, r.replacement.unwrap_or(AvatarStyle::DEFAULT));
            assert!(AvatarStyle::ALL.contains(&resolved));
            assert!(!r.reason.is_empty() && r.date.len() == 10 && r.retired_in == "0.2.0");
        }
        let mut ids: Vec<_> = RETIRED.iter().map(|r| r.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), RETIRED.len());
        assert_eq!(AvatarStyle::ALL.len() + RETIRED.len(), 44);
    }
    #[test]
    fn exactly_the_seven_rated_designs_are_ready_and_every_fallback_is_ready() {
        let mut ready: Vec<_> = AvatarStyle::ready().map(|s| s.id()).collect();
        ready.sort();
        assert_eq!(
            ready,
            [
                "iphone-weave",
                "watch-halo",
                "watch-hover",
                "watch-murmuration",
                "watch-rings",
                "watch-wave",
                "watch-well",
            ]
        );
        assert_eq!(AvatarStyle::all().count(), 27);
        assert_eq!(
            AvatarStyle::all()
                .filter(|s| s.status() == DesignStatus::Experimental)
                .count(),
            20
        );
        assert!(AvatarStyle::ready().all(|s| AvatarStyle::all().any(|a| a == s)));
        assert!(AvatarStyle::DEFAULT.is_ready());
        assert_eq!(AvatarStyle::default(), AvatarStyle::DEFAULT);
        for r in RETIRED {
            let target = AvatarStyle::resolve(r.id);
            assert!(target.is_ready(), "{} falls back to {}", r.id, target.id());
        }
        assert_eq!(AvatarStyle::resolve("eye"), AvatarStyle::WatchWell);
        assert_eq!(
            AvatarStyle::resolve("watch-shape-wave"),
            AvatarStyle::WatchWave
        );
        assert_eq!(AvatarStyle::resolve("watch-lens"), AvatarStyle::WatchHover);
        // Experimental designs stay reachable by id: nothing refuses them.
        assert_eq!(AvatarStyle::resolve("halo"), AvatarStyle::Halo);
        assert_eq!(
            AvatarStyle::from_id("watch-eye"),
            Some(AvatarStyle::WatchEye)
        );
    }
    #[cfg(feature = "json")]
    #[test]
    fn catalog_json_matches_the_compiled_catalog_and_retired_list() {
        let shipped: serde_json::Value =
            serde_json::from_str(include_str!("../catalog.json")).unwrap();
        assert_eq!(shipped["default"], AvatarStyle::DEFAULT.id());
        assert_eq!(shipped["schema_version"], 1);
        assert_eq!(shipped["schema_minor"], 1);
        let designs = shipped["designs"].as_array().unwrap();
        assert_eq!(designs.len(), AvatarStyle::ALL.len());
        for (row, style) in designs.iter().zip(AvatarStyle::ALL) {
            assert_eq!(row["code"], style.code());
            assert_eq!(row["id"], style.id());
            assert_eq!(row["name"], style.label());
            assert_eq!(row["gaze"], style.gaze());
            assert_eq!(row["status"], style.status().id());
        }
        let retired = shipped["retired"].as_array().unwrap();
        assert_eq!(retired.len(), RETIRED.len());
        for (row, r) in retired.iter().zip(RETIRED) {
            assert_eq!(row["id"], r.id);
            assert_eq!(
                row["replacement"],
                serde_json::json!(r.replacement.map(|s| s.id()))
            );
            assert_eq!(row["retired_in"], r.retired_in);
            assert_eq!(row["date"], r.date);
            assert_eq!(row["reason"], r.reason);
            assert!(
                designs.iter().all(|d| d["id"] != row["id"]),
                "{} is active",
                r.id
            );
        }
    }
    #[test]
    fn unknown_ids_fall_back_to_the_documented_default() {
        assert_eq!(AvatarStyle::DEFAULT, AvatarStyle::default());
        for id in ["", "future", "WATCH-HALO", "watch-face ", "\u{0}"] {
            assert_eq!(AvatarStyle::resolve(id), AvatarStyle::WatchHalo, "{id:?}");
        }
        for style in AvatarStyle::ALL {
            assert_eq!(AvatarStyle::resolve(style.id()), style);
        }
    }
}
