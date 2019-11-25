use super::request::*;

use std::cmp::Ordering;


/// HSB colour representation - hue, saturation, brightness (aka value).
/// Aka HSV (LIFX terminology) - hue, saturation, value.
/// This is not the same as HSL as used in CSS.
/// LIFX uses HSB aka HSV, not HSL.
#[derive(Debug)]
pub struct HSB {
    pub hue: u16,
    pub saturation: u8,
    pub brightness: u8,
}

///  RGB colour representation - red, green, blue.
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// HSBK colour representation - hue, saturation, brightness, kelvin.
/// Kelvin seems to be relevant only to whites - temperature of white.
#[derive(Debug)]
pub struct HSBK {
    pub hue: u16,
    pub saturation: u8,
    pub brightness: u8,
    pub kelvin: u16,
}

impl HSB {
    pub fn new(h: u16, s: u8, b: u8) -> HSB {
        HSB {
            hue: h,
            saturation: s,
            brightness: b,
        }
    }
}

impl From<HSBK> for HSB {
    fn from(c: HSBK) -> HSB {
        HSB::new(c.hue, c.saturation, c.brightness)
    }
}

/// The max value of the two byte representation of colour element as used in the protocol.
const WORD_SIZE: usize = 65535;
const DEGREES_UBOUND: usize = 360;
const PERCENT_UBOUND: usize = 100;

// (WORD_SIZE / DEGREES_UBOUND) is ~182.0417
// The two-byte represenation only represents integers, so decimals will be truncated.
// This can result in a get_state returning a slightly different result from the 
// preceding set_state for hue, saturation, and brightness.

pub fn hue_degrees_to_word(degrees: u16) -> [u8; 2] {
    let f = degrees as f64 * WORD_SIZE as f64 / DEGREES_UBOUND as f64;
    let b = RequestBin::u16_to_u8_array(f.round() as u16);
    [b[0], b[1]]
}

pub fn hue_word_to_degrees(word: u16) -> u16 {
    (word as usize * 360 / WORD_SIZE) as u16
}

pub fn saturation_percent_to_word(percent: u8) -> [u8; 2] {
    let f: f64 = percent as f64 * WORD_SIZE as f64 / 100.0;
    let b = RequestBin::u16_to_u8_array(f.round() as u16);
    [b[0], b[1]]
}

pub fn saturation_word_to_percent(word: u16) -> u8 {
    (word as usize * 100 / WORD_SIZE) as u8
}

pub fn brightness_percent_to_word(percent: u8) -> [u8; 2] {
    saturation_percent_to_word(percent)
}

pub fn brightness_word_to_percent(word: u16) -> u8 {
    (word as usize * 100 / WORD_SIZE) as u8
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
        assert_eq!([0x47, 0x1C], hue_degrees_to_word(100));
        assert_eq!([0x44, 0x44], hue_degrees_to_word(96));
        assert_eq!([0x43, 0x8E], hue_degrees_to_word(95));
    }

    #[test]
    fn test_hue_word_to_degrees() {
        assert_eq!(360, hue_word_to_degrees(65535));
        assert_eq!(0, hue_word_to_degrees(0));
        assert_eq!(180, hue_word_to_degrees(32768));
    }

    #[test]
    fn test_saturation_percent_to_word() {
        assert_eq!([0x80, 0x00], saturation_percent_to_word(50));
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

pub fn named_colours() -> Vec<String> {
    vec!(
        "beige".to_string(), 
        "blue".to_string(), 
        "chartreuse".to_string(),
        "coral".to_string(), 
        "cornflower".to_string(),
        "crimson".to_string(), 
        "deep_sky_blue".to_string(),
        "green".to_string(), 
        "red".to_string(), 
        "slate_gray".to_string(), 
    )
}

pub fn get_colour(s: &str) -> HSB {
    let colour: &str = &(s.to_lowercase());
    match colour {
        "beige" => {
            HSB {
                hue: 60,
                saturation: 56,
                brightness: 91,
            }
        }
        "blue" => {
            HSB {
                hue: 240,
                saturation: 100,
                brightness: 50,
            }
        }
        "chartreuse" => {
            HSB {
                hue: 90,
                saturation: 100,
                brightness: 50,
            }
        }
        "coral" => {
            HSB {
                hue: 16,
                saturation: 100,
                brightness: 66,
            }
        }
        "cornflower" => {
            HSB {
                hue: 219,
                saturation: 79,
                brightness: 66,
            }
        }
        "crimson" => {
            HSB {
                hue: 348,
                saturation: 83,
                brightness: 47,
            }
        }
        "deep_sky_blue" => {
            HSB {
                hue: 195,
                saturation: 100,
                brightness: 50,
            }
        }
        "green" => {
            HSB {
                hue: 120,
                saturation: 100,
                brightness: 50,
            }
        }
        "red" => {
            HSB {
                hue: 0,
                saturation: 100,
                brightness: 50,
            }
        }
        "slate_gray" => {
            HSB {
                hue: 210,
                saturation: 13,
                brightness: 50,
            }
        }
        _ => panic!("no such colour."),
    }

}
