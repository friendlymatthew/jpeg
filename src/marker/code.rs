use crate::marker::{Marker, MarkerType};
use std::collections::HashSet;

impl Marker {

    pub(crate) const SIZE: usize = 2;
    pub fn all() -> HashSet<Marker> {
        use crate::marker::Marker::*;

        HashSet::from([
            SOF0, SOF1, SOF2, SOF3, DHT, SOF5, SOF6, SOF7, JPG, SOF9, SOF10, SOF11, DAC, SOF13,
            SOF14, SOF15, RST0, RST1, RST2, RST3, RST4, RST5, RST6, RST7, SOI, EOI, SOS, DQT, DNL,
            DRI, DHP, EXP, APP0, APP1, APP2, APP3, APP4, APP5, APP6, APP7, APP8, APP9, APPA, APPB,
            APPC, APPD, APPE, APPF, JPEG0, JPEG1, JPEG2, JPEG3, JPEG4, JPEG5, JPEG6, JPEG7, JPEG8,
            JPEG9, JPEG10, JPEG11, JPEG12, JPEG13, COM, TEM, RES2, RES3, RES4, RES5, RES6, RES7,
            RES8, RES9, RES10, RES11, RES12, RES13, RES14, RES15, RES16, RES17, RES18, RES19,
            RES20, RES21, RES22, RES23, RES24, RES25, RES26, RES27, RES28, RES29, RES30, RES31,
            RES32, RES33, RES34, RES35, RES36, RES37, RES38, RES39, RES40, RES41, RES42, RES43,
            RES44, RES45, RES46, RES47, RES48, RES49, RES50, RES51, RES52, RES53, RES54, RES55,
            RES56, RES57, RES58, RES59, RES60, RES61, RES62, RES63, RES64, RES65, RES66, RES67,
            RES68, RES69, RES70, RES71, RES72, RES73, RES74, RES75, RES76, RES77, RES78, RES79,
            RES80, RES81, RES82, RES83, RES84, RES85, RES86, RES87, RES88, RES89, RES90, RES91,
            RES92, RES93, RES94, RES95, RES96, RES97, RES98, RES99, RES100, RES101, RES102, RES103,
            RES104, RES105, RES106, RES107, RES108, RES109, RES110, RES111, RES112, RES113, RES114,
            RES115, RES116, RES117, RES118, RES119, RES120, RES121, RES122, RES123, RES124, RES125,
            RES126, RES127, RES128, RES129, RES130, RES131, RES132, RES133, RES134, RES135, RES136,
            RES137, RES138, RES139, RES140, RES141, RES142, RES143, RES144, RES145, RES146, RES147,
            RES148, RES149, RES150, RES151, RES152, RES153, RES154, RES155, RES156, RES157, RES158,
            RES159, RES160, RES161, RES162, RES163, RES164, RES165, RES166, RES167, RES168, RES169,
            RES170, RES171, RES172, RES173, RES174, RES175, RES176, RES177, RES178, RES179, RES180,
            RES181, RES182, RES183, RES184, RES185, RES186, RES187, RES188, RES189, RES190, RES191,
        ])
    }

    pub fn is_invalid_low_byte(&self) -> bool {
        match self {
            Marker::GLOBAL | Marker::STUFF => true,
            _ => false,
        }
    }

    pub fn singleton(&self) -> bool {
        match self {
            Marker::DHT => false,
            Marker::DAC => false,
            Marker::DQT => false,
            Marker::EXP => false,
            _ => true,
        }
    }

    /// Some markers stand alone, that is, which is not the start of a marker segment.
    pub fn is_segment(&self) -> MarkerType {
        match self {
            Marker::RST0
            | Marker::RST1
            | Marker::RST2
            | Marker::RST3
            | Marker::RST4
            | Marker::RST5
            | Marker::RST6
            | Marker::RST7
            | Marker::SOI
            | Marker::EOI
            | Marker::TEM => MarkerType::StandAlone,
            _ => MarkerType::Segment,
        }
    }

    pub fn to_u16(&self) -> u16 {
        u16::from_be_bytes([Marker::GLOBAL as u8, *self as u8])
    }
}
