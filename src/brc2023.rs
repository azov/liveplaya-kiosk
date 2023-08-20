use crate::{brc::*, util::geo::Point, util::time::Timestamp};

const BRC2023_CENTERLINES: &'static str = std::include_str!("../data/2023/Street_Centerlines.json");
const BRC2023_OUTLINES: &'static str = std::include_str!("../data/2023/Street_Outlines.json");
const BRC2023_EXTENT: &'static str = std::include_str!("../data/2023/City_Extent.json");

pub fn get() -> BlackRockCity {
    let gates_open = Timestamp::from_calendar_pdt(2023, 8, 27, 0, 0, 0).unwrap();
    let golden_stake = Point::new(-119.2035, 40.7864).unwrap();
    let centerlines = serde_json::from_str(BRC2023_CENTERLINES).unwrap();
    let outlines = serde_json::from_str(BRC2023_OUTLINES).unwrap();
    let extent = serde_json::from_str(BRC2023_EXTENT).unwrap();

    BlackRockCity::from_bmorg_data(gates_open, golden_stake, 45., extent, centerlines, outlines)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let brc = get();
        assert_eq!(
            brc.ring_by_name("d").unwrap().name().to_owned(),
            "Dingbat"
        );
        assert_eq!(
            brc.ring_by_name("k").unwrap().name().to_owned(),
            "Kraken"
        );
    }

    #[test]
    fn test_rgeocode() {
        let brc = get();
        assert_eq!(
            brc.rgeocode(Point::new(-119.195238274, 40.7801310097).unwrap()),
            "3:00 & C" //it's actually "3 & B Plaza", but the location is really closer to C?..
        );
    }
}
