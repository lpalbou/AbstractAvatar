//! Stable procedural design catalog. Numeric ABI codes are append-only; persist string ids.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum AvatarStyle {
    #[default]
    Halo,
    Hover,
    Vortex,
    Eclipse,
    Eye,
    Rover,
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
    WatchDrop,
    WatchWave,
    WatchWell,
    WatchBlackHole,
    WatchAccretion,
    WatchFace,
    WatchImprint,
    WatchContour,
    WatchMembrane,
    WatchAurora,
    WatchMurmuration,
    WatchEye,
    WatchLens,
    WatchDome,
    WatchValet,
    WatchRetro,
    WatchDeco,
    WatchRover,
    WatchHover,
    WatchSpark,
    WatchCruiser,
    WatchAndroid,
    WatchSentinel,
    WatchOcean,
    WatchKitty,
    WatchFaceShape,
    WatchWaveShape,
    IphoneWeave,
}
impl AvatarStyle {
    pub const ALL: [Self; 44] = [
        Self::Halo,
        Self::Hover,
        Self::Vortex,
        Self::Eclipse,
        Self::Eye,
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
        Self::WatchDrop,
        Self::WatchWave,
        Self::WatchWell,
        Self::WatchBlackHole,
        Self::WatchAccretion,
        Self::WatchFace,
        Self::WatchImprint,
        Self::WatchContour,
        Self::WatchMembrane,
        Self::WatchAurora,
        Self::WatchMurmuration,
        Self::WatchEye,
        Self::WatchLens,
        Self::WatchDome,
        Self::WatchValet,
        Self::WatchRetro,
        Self::WatchDeco,
        Self::WatchRover,
        Self::WatchHover,
        Self::WatchSpark,
        Self::WatchCruiser,
        Self::WatchAndroid,
        Self::WatchSentinel,
        Self::WatchOcean,
        Self::WatchKitty,
        Self::WatchFaceShape,
        Self::WatchWaveShape,
        Self::IphoneWeave,
    ];
    pub fn from_code(code: u32) -> Option<Self> {
        Self::ALL.get(code as usize).copied()
    }
    pub fn code(self) -> u32 {
        Self::ALL.iter().position(|v| *v == self).unwrap() as u32
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
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
    pub fn fields(self) -> [&'static str; 5] {
        match self{
        Self::Halo => ["halo", "Halo orb", "A luminous corona", "A", "orbs"],
        Self::Hover => ["hover", "Hover robot", "Warm eyes and floating thrusters", "HV", "machines"],
        Self::Vortex => ["vortex", "Vortex", "Spiral arms gather to listen and reach out to speak", "B", "orbs"],
        Self::Eclipse => ["eclipse", "Eclipse", "A dark disc, golden corona and working rays", "U", "forms"],
        Self::Eye => ["eye", "Eye", "A watchful eye that opens to listen and scans to work", "K", "faces"],
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
        Self::WatchDrop => ["watch-drop", "Watch Drop", "a drop of light: a vortex to think, fibres to work, a horn of particles to speak", "W", "forms"],
        Self::WatchWave => ["watch-wave", "Watch Wave", "ribbons of light, each a harmonic of the voice; they wind into a swirl to work", "X", "forms"],
        Self::WatchWell => ["watch-well", "Watch Well", "space bending into a well: ripples carry the voice, two masses circle it as it works", "T", "cosmos"],
        Self::WatchBlackHole => ["watch-black-hole", "Watch Black hole", "a ring of light round a shadow: gas falls in to listen, a disk to think, jets to work", "Y", "cosmos"],
        Self::WatchAccretion => ["watch-accretion", "Watch Accretion", "a young star's disk of dust: it gathers into clumps to think and merges into planets to work", "Z", "cosmos"],
        Self::WatchFace => ["watch-face", "Watch Face", "a friendly face whose lips follow its voice; it cups its ear to listen", "G", "faces"],
        Self::WatchImprint => ["watch-imprint", "Watch Imprint", "a face seen only where it touches a pane of glass; it presses in as it speaks", "I", "faces"],
        Self::WatchContour => ["watch-contour", "Watch Contour", "a face raised in lines of light: its lips speak, the words leave them as waves along the lines", "CT", "faces"],
        Self::WatchMembrane => ["watch-membrane", "Watch Membrane", "a face pressing through a fabric of light: it presses closer with the voice and speaks through it", "MB", "faces"],
        Self::WatchAurora => ["watch-aurora", "Watch Aurora", "the northern lights over a lake: they rise with the voice and gather into a corona to think", "AU", "nature"],
        Self::WatchMurmuration => ["watch-murmuration", "Watch Murmuration", "a flock of starlings at dusk: it swells to listen, spins to think, streams to work, draws the voice", "MU", "nature"],
        Self::WatchEye => ["watch-eye", "Watch Eye", "one friendly eye: its iris glows with its voice, it opens wide to listen", "K", "faces"],
        Self::WatchLens => ["watch-lens", "Watch Lens", "a glowing red camera eye; its core flares as it speaks", "L", "machines"],
        Self::WatchDome => ["watch-dome", "Watch Dome", "a dome-headed droid: it turns to face you, chatters with its lights, spins to think", "M", "machines"],
        Self::WatchValet => ["watch-valet", "Watch Valet", "a fussy gilded valet, stiff and proper: it frets as it thinks and talks with its hands", "N", "machines"],
        Self::WatchRetro => ["watch-retro", "Watch Retro", "a gentle giant of the fifties, slow and deliberate: the works in its dome whirr as it thinks, neon tubes flicker as it talks", "O", "machines"],
        Self::WatchDeco => ["watch-deco", "Watch Deco", "a serene art-deco machine woman, regal and unhurried, rings of light passing around her", "P", "machines"],
        Self::WatchRover => ["watch-rover", "Watch Rover", "a shy, curious little rover on tracks: its eyes lean in to listen, tilt when puzzled, bob when it talks", "Q", "machines"],
        Self::WatchHover => ["watch-hover", "Watch Hover", "a round floating robot, polite and brave: its red eyes glow with its voice, it drifts closer to listen", "HV", "machines"],
        Self::WatchSpark => ["watch-spark", "Watch Spark", "a curious robot on tracks with shutter eyes: it leans in for input and its eyes flick while it thinks", "SP", "machines"],
        Self::WatchCruiser => ["watch-cruiser", "Watch Cruiser", "a sleek black car: its red scanner sweeps while it thinks, its dash lights up with its voice", "CR", "machines"],
        Self::WatchAndroid => ["watch-android", "Watch Android", "a pale gold android with amber eyes: a tilt of the head while it thinks, a precise blink", "AN", "machines"],
        Self::WatchSentinel => ["watch-sentinel", "Watch Sentinel", "a chrome sentinel helmet whose red eye sweeps side to side, faster as it thinks; it speaks in a low glow", "CY", "machines"],
        Self::WatchOcean => ["watch-ocean", "Watch Ocean", "a sea at dusk whose swell rises with its voice; while it thinks the waves fall into order", "H", "nature"],
        Self::WatchKitty => ["watch-shape-kitty", "Kitty silhouette", "A point-cloud cat with two ears, responding to activity and voice", "shape:kitty", "faces"],
        Self::WatchFaceShape => ["watch-shape-face", "Face silhouette", "The original point-cloud smiling face silhouette", "shape:face", "faces"],
        Self::WatchWaveShape => ["watch-shape-wave", "Wave silhouette", "The original planar point-cloud wave silhouette", "shape:wave", "forms"],
        Self::IphoneWeave => ["iphone-weave", "Woven orb", "Five cords become a geodesic sphere or a raised approval hand", "", "forms"],
    }
    }
}
