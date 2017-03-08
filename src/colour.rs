use super::request::*;

use std::cmp::Ordering;


#[derive(Debug)]
pub struct HSB {
    pub hue: u16,
    pub saturation: u8,
    pub brightness: u8,
}

pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl HSB {
    fn new(h: u16, s: u8, b: u8) -> HSB {
        HSB {
            hue: h,
            saturation: s,
            brightness: b,
        }
    }
}

const WORDSIZE: f32 = 65535.0;

// LIFX uses HSB aka HSV, not HSL.

pub fn hue_degrees_to_word(degrees: u16) -> [u8; 2] {
    let f = degrees as f32 / 360.0 * WORDSIZE;
    let b = RequestBin::u16_to_u8_array(f as u16);
    [b[0], b[1]]
}

pub fn saturation_percent_to_word(percent: u8) -> [u8; 2] {
    let f = percent as f32 / 100.0 * WORDSIZE;
    let b = RequestBin::u16_to_u8_array(f as u16);
    [b[0], b[1]]
}

pub fn brightness_percent_to_word(percent: u8) -> [u8; 2] {
    saturation_percent_to_word(percent)
}

pub fn rgb_to_hsv(rgb: RGB) -> HSB {
    let r1 = rgb.red as f32 / 255.0;
    let g1 = rgb.green as f32 / 255.0;
    let b1 = rgb.blue as f32 / 255.0;

    let mut floats: Vec<f32> = vec![r1, g1, b1];
    floats.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let cmax = floats[2];
    let cmin = floats[0];
    let d = cmax - cmin;

    // Hue.
    let h = match cmax {
        _ if r1 == cmax => (((g1 - b1) / d) % 6.0) * 60.0,
        _ if g1 == cmax => (((b1 - r1) / d) + 2.0) * 60.0,
        _ if b1 == cmax => (((r1 - g1) / d) + 4.0) * 60.0,
        _ => 0.0,
    };

    // Saturation.
    let s = match cmax {
        0.0 => 0.0,
        _ => d / cmax,
    };

    // Value / brightness.
    let v = cmax;

    HSB {
        hue: h as u16,
        saturation: (s * 100.0) as u8,
        brightness: (v * 100.0) as u8,
    }
}

#[cfg(test)]
mod tests {

    use colour::*;

    #[test]
    fn test_hue_degrees_to_word() {
        assert_eq!([0x55, 0x55], hue_degrees_to_word(120));
    }

    #[test]
    fn test_saturation_percent_to_word() {
        assert_eq!([0x7F, 0xFF], saturation_percent_to_word(50));
    }

    #[test]
    fn test_rgb_to_hsv() {
        struct Test {
            rgb: RGB,
            hsb: HSB,
        };

        let tests = vec![
                        Test {
                             rgb: RGB { // olive
                                 red: 128,
                                 green: 128,
                                 blue: 0,
                             },
                             hsb: HSB {
                                 hue: 60,
                                 saturation: 100,
                                 brightness: 50,
                             },
                         },
                         Test {
                             rgb: RGB { // chartreuse
                                 red: 127,
                                 green: 255,
                                 blue: 0,
                             },
                             hsb: HSB {
                                 hue: 90,
                                 saturation: 100,
                                 brightness: 100,
                             },
                         },
        ];

        for t in tests {
            let res = rgb_to_hsv(t.rgb);
            assert_eq!(res.hue, t.hsb.hue);
            assert_eq!(res.saturation, t.hsb.saturation);
            assert_eq!(res.brightness, t.hsb.brightness);
        }
    }
}

pub const BEIGE: HSB = HSB {
    hue: 60,
    saturation: 56,
    brightness: 91,
};
pub const BLUE: HSB = HSB {
    hue: 240,
    saturation: 100,
    brightness: 50,
};
pub const CHARTREUSE: HSB = HSB {
    hue: 90,
    saturation: 100,
    brightness: 50,
};
pub const CORAL: HSB = HSB {
    hue: 16,
    saturation: 100,
    brightness: 66,
};
pub const CORNFLOWER: HSB = HSB {
    hue: 219,
    saturation: 79,
    brightness: 66,
};
pub const CRIMSON: HSB = HSB {
    hue: 348,
    saturation: 83,
    brightness: 47,
};
pub const DEEP_SKY_BLUE: HSB = HSB {
    hue: 195,
    saturation: 100,
    brightness: 50,
};
pub const GREEN: HSB = HSB {
    hue: 120,
    saturation: 100,
    brightness: 50,
};
pub const RED: HSB = HSB {
    hue: 0,
    saturation: 100,
    brightness: 50,
};
pub const SLATE_GRAY: HSB = HSB {
    hue: 210,
    saturation: 13,
    brightness: 50,
};
