pub fn ft2m(feet: f64) -> f64 {
    feet * 0.3048
}

pub fn m2ft(meters: f64) -> f64 {
    meters / ft2m(1.)
}
