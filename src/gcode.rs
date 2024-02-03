#[derive(Debug, PartialEq)]
pub enum Gcode {
    // G1 P100 T200
    G1Move(Option<f32>, Option<f32>),
    // G28
    G28Home,
    // G90
    G90SetAbsolute,
    // G91
    G91SetRelative,
    // M1 P0.5
    // M1 T1.5
    // M1 T1.5 P20
    M1SetVelocity(Option<f32>, Option<f32>),
}

pub fn invalid_gcode(str: &str) -> String {
    format!("invalid gcode: {}", str)
}

fn get_pan_tilt_floats(parts: &[(Option<char>, f32)]) -> (Option<f32>, Option<f32>) {
    let pan = parts
        .iter()
        .find(|(char_opt, _)| char_opt.unwrap_or(' ') == 'P')
        .map(|(_, n)| n.to_owned());
    let tilt = parts
        .iter()
        .find(|(char_opt, _)| char_opt.unwrap_or(' ') == 'T')
        .map(|(_, n)| n.to_owned());
    (pan, tilt)
}

pub struct GcodeParser;

impl GcodeParser {
    pub fn of_str(str: &str) -> Result<Gcode, String> {
        let mut parts = str.split_whitespace().try_fold(vec![], |mut acc, part| {
            let mut chars = part.chars();
            let start_char = chars.next();
            let num = chars.as_str().parse::<f32>();
            match (start_char, num) {
                (Some('G') | Some('M') | Some('T') | Some('P'), Ok(num)) => {
                    acc.push((start_char, num));
                    Ok(acc)
                }
                (Some(_), _) => Err(invalid_gcode(str)),
                (None, _) => part
                    .parse::<f32>()
                    .map(|num| {
                        acc.push((None, num));
                        acc
                    })
                    .map_err(|_| invalid_gcode(str)),
            }
        })?;
        parts.reverse();
        let (first_char, fist_num) = parts.pop().ok_or_else(|| invalid_gcode(str))?;

        // cast to suppress stupid rust warning on issue no one is really even
        // sure if they're going to take action on.
        match (first_char, fist_num as u32) {
            (Some('G'), 1) => {
                let (pan, tilt) = get_pan_tilt_floats(&parts);
                Ok(Gcode::G1Move(pan, tilt))
            }
            (Some('G'), 28) => Ok(Gcode::G28Home),
            (Some('G'), 90) => Ok(Gcode::G90SetAbsolute),
            (Some('G'), 91) => Ok(Gcode::G91SetRelative),
            (Some('M'), 1) => {
                let (pan, tilt) = get_pan_tilt_floats(&parts);
                Ok(Gcode::M1SetVelocity(pan, tilt))
            }
            _ => Err(invalid_gcode(str)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g1_move() {
        let gcode = GcodeParser::of_str("G1 P50 T60").unwrap();
        assert_eq!(gcode, Gcode::G1Move(Some(50.0), Some(60.0)));
    }

    #[test]
    fn test_g28_home() {
        let gcode = GcodeParser::of_str("G28").unwrap();
        assert_eq!(gcode, Gcode::G28Home);
    }

    #[test]
    fn test_m1_set_velocity() {
        let gcode = GcodeParser::of_str("M1 T2000.1  P1000").unwrap();
        assert_eq!(gcode, Gcode::M1SetVelocity(Some(1000.0), Some(2000.1)));
    }
}
