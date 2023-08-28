use crate::{brc::*, util::geo::Point, util::time::Timestamp, io::site::Poi};

const BRC2023_CENTERLINES: &'static str = std::include_str!("../data/2023/Street_Centerlines.json");
const BRC2023_OUTLINES: &'static str = std::include_str!("../data/2023/Street_Outlines.json");
const BRC2023_EXTENT: &'static str = std::include_str!("../data/2023/City_Extent.json");


pub const POIS : [Poi; 13] = [
    Poi{slug: "tgecko", mobile: "vehicle", name: "Techno Gecko", call: "TGECKO", is_favorite: true},
    Poi{slug: "dfguppy",  mobile: "vehicle", name: "Guppy", call: "DFGUPY",  is_favorite: true},
    Poi{slug: "dfkeggy",  mobile: "vehicle", name: "Keggy", call: "DFKEGY", is_favorite: true},
    Poi{slug: "duck",  mobile: "vehicle", name: "The Duck", call: "DUCK",  is_favorite: false},
    Poi{slug: "moebius-omnibus-1",  mobile: "vehicle", name: "Moebius Omnibus", call: "K6CQU-4", is_favorite: false},
    Poi{slug: "moebius-omnibus",  mobile: "vehicle", name: "Moebius Omnibus", call: "MOBIUS", is_favorite: false},
    Poi{slug: "raptor",  mobile: "vehicle", name: "Raptor", call: "MOBIUS-1", is_favorite: false},
    Poi{slug: "k9ivbm",  mobile: "vehicle", name: "K9 Mark IV-BM", call: "K9IVBM", is_favorite: false},
    Poi{slug: "peef",   mobile: "person", name: "pEEf", call: "PEEF", is_favorite: false},
    Poi{slug: "gibbon", mobile: "person", name: "Gibbon", call: "GIBBON-1", is_favorite: false},
    Poi{slug: "gibbon", mobile: "person", name: "Rebecky", call: "KK6UXV-9", is_favorite: false},
    Poi{slug: "k6cqu-5", mobile: "nope", name: "PlayaMap.org", call: "K6CQU-5", is_favorite: false}, 
    Poi{slug: "brcwx", mobile: "nope", name: "BRC Weather Station", call: "BRCWX", is_favorite: false},
    
];


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
