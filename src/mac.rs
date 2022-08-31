use std::{error::Error, fmt::Display, str::FromStr};

/// MAC suitable usage with BGX13P commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mac([u8; 6]);

impl Display for Mac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl FromStr for Mac {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace(':', "").to_lowercase();
        if s.len() == 12 {
            Ok(Self([
                u8::from_str_radix(s.get(..2).ok_or("Couldn't get index from radix str")?, 16)?,
                u8::from_str_radix(s.get(2..4).ok_or("Couldn't get index from radix str")?, 16)?,
                u8::from_str_radix(s.get(4..6).ok_or("Couldn't get index from radix str")?, 16)?,
                u8::from_str_radix(s.get(6..8).ok_or("Couldn't get index from radix str")?, 16)?,
                u8::from_str_radix(s.get(8..10).ok_or("Couldn't get index from radix str")?, 16)?,
                u8::from_str_radix(s.get(10..).ok_or("Couldn't get index from radix str")?, 16)?,
            ]))
        } else {
            Err("Wrong size of MAC address".into())
        }
    }
}

#[test]
fn mac_parse_1() {
    const MAC_str: &str = "d0:cf:5e:82:85:06";
    const MAC: Mac = Mac([0xd0, 0xcf, 0x5e, 0x82, 0x85, 0x06]);

    let mac_str = MAC_str.parse().unwrap();

    assert_eq!(MAC, mac_str)
}
