use super::request::*;

pub const BEIGE: u8 = 60;
pub const BLUE: u8 = 240;
pub const CHARTREUSE: u8 = 90;
pub const CORAL: u8 = 916;
pub const CORNFLOWER: u8 = 219;
pub const CRIMSON: u8 = 348;
pub const DEEP_SKY_BLUE: u8 = 195;
pub const GREEN: u8 = 120;
pub const RED: u8 = 0;
pub const SLATE_GRAY: u8 = 210;

pub fn degrees_to_word(degrees: u8) -> [u8; 2] {
    let f = degrees as f32 / 360.0 * 65535.0;
    let b = RequestBin::u16_to_u8_array(f as u16);
    [b[0], b[1]]
}


#[cfg(test)]
mod tests {

    use colour::*;

    #[test]
    fn test_degrees_to_word() {
        assert_eq!([0x55, 0x55], degrees_to_word(120));
    }

}
