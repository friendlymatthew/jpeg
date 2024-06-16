use std::collections::HashSet;

use crate::coding::{CodingProcess, EntropyCoding};

pub(crate) enum MarkerType {
    Segment,
    StandAlone,
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Hash, Eq)]
pub(crate) enum Marker {
    GLOBAL = 0xFF,
    STUFF = 0x00,

    /// Start of Frame markers, non-differential, Huffman coding
    /// Baseline DCT
    SOF0 = 0xC0,

    /// Extended sequential DCT
    SOF1 = 0xC1,

    /// Progressive DCT
    SOF2 = 0xC2,

    /// Lossless (sequential)
    SOF3 = 0xC3,

    /// Start of Frame markers, differential, Huffman coding
    /// Differential sequential DCT
    SOF5 = 0xC5,

    /// Differential progressive DCT
    SOF6 = 0xC6,

    /// Differential lossless (sequential)
    SOF7 = 0xC7,

    /// Start of Frame markers, non-differential, arithmetic coding
    JPG = 0xC8,

    /// Extended Sequential DCT
    SOF9 = 0xC9,

    /// Progressive DCT
    SOF10 = 0xCA,

    /// Lossless (sequential)
    SOF11 = 0xCB,

    /// Start of Frame markers, differential, arithmetic coding

    /// Differential sequential DCT
    SOF13 = 0xCD,

    /// Differential progressive DCT
    SOF14 = 0xCE,

    /// Differential lossless (sequential)
    SOF15 = 0xCF,

    /// Huffman table specification
    DHT = 0xC4,

    /// Arithmetic coding condition specification
    /// Define arithmetic coding conditioning(s)
    DAC = 0xCC,

    /// Restart interval termination
    /// Restart with modulo 8 count "M"
    /// Restart Marker 0
    RST0 = 0xD0,
    /// Restart Marker 1
    RST1 = 0xD1,
    /// Restart Marker 2
    RST2 = 0xD2,
    /// Restart Marker 3
    RST3 = 0xD3,
    /// Restart Marker 4
    RST4 = 0xD4,
    /// Restart Marker 5
    RST5 = 0xD5,
    /// Restart Marker 6
    RST6 = 0xD6,
    /// Restart Marker 7
    RST7 = 0xD7,

    /// Start of image
    SOI = 0xD8,

    /// End of image
    EOI = 0xD9,

    /// Start of scan
    SOS = 0xDA,

    /// Define quantization table(s)
    DQT = 0xDB,

    /// Define number of lines
    DNL = 0xDC,

    /// Define restart interval
    DRI = 0xDD,

    /// Define hierarchical progression
    DHP = 0xDE,

    /// Expand reference components
    EXP = 0xDF,

    /// Reserved for application segments
    /// Application 0
    APP0 = 0xE0,
    /// Application 1
    APP1 = 0xE1,
    /// Application 2
    APP2 = 0xE2,
    /// Application 3
    APP3 = 0xE3,
    /// Application 4
    APP4 = 0xE4,
    /// Application 5
    APP5 = 0xE5,
    /// Application 6
    APP6 = 0xE6,
    /// Application 7
    APP7 = 0xE7,
    /// Application 8
    APP8 = 0xE8,
    /// Application 9
    APP9 = 0xE9,
    /// Application 10
    APPA = 0xEA,
    /// Application 11
    APPB = 0xEB,
    /// Application 12
    APPC = 0xEC,
    /// Application 13
    APPD = 0xED,
    /// Application 14
    APPE = 0xEE,
    /// Application 15
    APPF = 0xEF,

    /// Reserved for JPEGn extensions
    /// JPEG 0
    JPEG0 = 0xF0,
    /// JPEG 1
    JPEG1 = 0xF1,
    /// JPEG 2
    JPEG2 = 0xF2,
    /// JPEG 3
    JPEG3 = 0xF3,
    /// JPEG 4
    JPEG4 = 0xF4,
    /// JPEG 5
    JPEG5 = 0xF5,
    /// JPEG 6
    JPEG6 = 0xF6,
    /// JPEG 7
    JPEG7 = 0xF7,
    /// JPEG 8
    JPEG8 = 0xF8,
    /// JPEG 9
    JPEG9 = 0xF9,
    /// JPEG 10
    JPEG10 = 0xFA,
    /// JPEG 11
    JPEG11 = 0xFB,
    /// JPEG 12
    JPEG12 = 0xFC,
    /// JPEG 13
    JPEG13 = 0xFD,

    /// Comment
    COM = 0xFE,

    // RESERVED MARKERS
    /// * For temporary private use in arithmetic coding
    TEM = 0x01,

    /// Reserved markers from 0x02 to 0xBF
    RES2 = 0xFF02,
    RES3 = 0xFF03,
    RES4 = 0xFF04,
    RES5 = 0xFF05,
    RES6 = 0xFF06,
    RES7 = 0xFF07,
    RES8 = 0xFF08,
    RES9 = 0xFF09,
    RES10 = 0xFF0A,
    RES11 = 0xFF0B,
    RES12 = 0xFF0C,
    RES13 = 0xFF0D,
    RES14 = 0xFF0E,
    RES15 = 0xFF0F,
    RES16 = 0xFF10,
    RES17 = 0xFF11,
    RES18 = 0xFF12,
    RES19 = 0xFF13,
    RES20 = 0xFF14,
    RES21 = 0xFF15,
    RES22 = 0xFF16,
    RES23 = 0xFF17,
    RES24 = 0xFF18,
    RES25 = 0xFF19,
    RES26 = 0xFF1A,
    RES27 = 0xFF1B,
    RES28 = 0xFF1C,
    RES29 = 0xFF1D,
    RES30 = 0xFF1E,
    RES31 = 0xFF1F,
    RES32 = 0xFF20,
    RES33 = 0xFF21,
    RES34 = 0xFF22,
    RES35 = 0xFF23,
    RES36 = 0xFF24,
    RES37 = 0xFF25,
    RES38 = 0xFF26,
    RES39 = 0xFF27,
    RES40 = 0xFF28,
    RES41 = 0xFF29,
    RES42 = 0xFF2A,
    RES43 = 0xFF2B,
    RES44 = 0xFF2C,
    RES45 = 0xFF2D,
    RES46 = 0xFF2E,
    RES47 = 0xFF2F,
    RES48 = 0xFF30,
    RES49 = 0xFF31,
    RES50 = 0xFF32,
    RES51 = 0xFF33,
    RES52 = 0xFF34,
    RES53 = 0xFF35,
    RES54 = 0xFF36,
    RES55 = 0xFF37,
    RES56 = 0xFF38,
    RES57 = 0xFF39,
    RES58 = 0xFF3A,
    RES59 = 0xFF3B,
    RES60 = 0xFF3C,
    RES61 = 0xFF3D,
    RES62 = 0xFF3E,
    RES63 = 0xFF3F,
    RES64 = 0xFF40,
    RES65 = 0xFF41,
    RES66 = 0xFF42,
    RES67 = 0xFF43,
    RES68 = 0xFF44,
    RES69 = 0xFF45,
    RES70 = 0xFF46,
    RES71 = 0xFF47,
    RES72 = 0xFF48,
    RES73 = 0xFF49,
    RES74 = 0xFF4A,
    RES75 = 0xFF4B,
    RES76 = 0xFF4C,
    RES77 = 0xFF4D,
    RES78 = 0xFF4E,
    RES79 = 0xFF4F,
    RES80 = 0xFF50,
    RES81 = 0xFF51,
    RES82 = 0xFF52,
    RES83 = 0xFF53,
    RES84 = 0xFF54,
    RES85 = 0xFF55,
    RES86 = 0xFF56,
    RES87 = 0xFF57,
    RES88 = 0xFF58,
    RES89 = 0xFF59,
    RES90 = 0xFF5A,
    RES91 = 0xFF5B,
    RES92 = 0xFF5C,
    RES93 = 0xFF5D,
    RES94 = 0xFF5E,
    RES95 = 0xFF5F,
    RES96 = 0xFF60,
    RES97 = 0xFF61,
    RES98 = 0xFF62,
    RES99 = 0xFF63,
    RES100 = 0xFF64,
    RES101 = 0xFF65,
    RES102 = 0xFF66,
    RES103 = 0xFF67,
    RES104 = 0xFF68,
    RES105 = 0xFF69,
    RES106 = 0xFF6A,
    RES107 = 0xFF6B,
    RES108 = 0xFF6C,
    RES109 = 0xFF6D,
    RES110 = 0xFF6E,
    RES111 = 0xFF6F,
    RES112 = 0xFF70,
    RES113 = 0xFF71,
    RES114 = 0xFF72,
    RES115 = 0xFF73,
    RES116 = 0xFF74,
    RES117 = 0xFF75,
    RES118 = 0xFF76,
    RES119 = 0xFF77,
    RES120 = 0xFF78,
    RES121 = 0xFF79,
    RES122 = 0xFF7A,
    RES123 = 0xFF7B,
    RES124 = 0xFF7C,
    RES125 = 0xFF7D,
    RES126 = 0xFF7E,
    RES127 = 0xFF7F,
    RES128 = 0xFF80,
    RES129 = 0xFF81,
    RES130 = 0xFF82,
    RES131 = 0xFF83,
    RES132 = 0xFF84,
    RES133 = 0xFF85,
    RES134 = 0xFF86,
    RES135 = 0xFF87,
    RES136 = 0xFF88,
    RES137 = 0xFF89,
    RES138 = 0xFF8A,
    RES139 = 0xFF8B,
    RES140 = 0xFF8C,
    RES141 = 0xFF8D,
    RES142 = 0xFF8E,
    RES143 = 0xFF8F,
    RES144 = 0xFF90,
    RES145 = 0xFF91,
    RES146 = 0xFF92,
    RES147 = 0xFF93,
    RES148 = 0xFF94,
    RES149 = 0xFF95,
    RES150 = 0xFF96,
    RES151 = 0xFF97,
    RES152 = 0xFF98,
    RES153 = 0xFF99,
    RES154 = 0xFF9A,
    RES155 = 0xFF9B,
    RES156 = 0xFF9C,
    RES157 = 0xFF9D,
    RES158 = 0xFF9E,
    RES159 = 0xFF9F,
    RES160 = 0xFFA0,
    RES161 = 0xFFA1,
    RES162 = 0xFFA2,
    RES163 = 0xFFA3,
    RES164 = 0xFFA4,
    RES165 = 0xFFA5,
    RES166 = 0xFFA6,
    RES167 = 0xFFA7,
    RES168 = 0xFFA8,
    RES169 = 0xFFA9,
    RES170 = 0xFFAA,
    RES171 = 0xFFAB,
    RES172 = 0xFFAC,
    RES173 = 0xFFAD,
    RES174 = 0xFFAE,
    RES175 = 0xFFAF,
    RES176 = 0xFFB0,
    RES177 = 0xFFB1,
    RES178 = 0xFFB2,
    RES179 = 0xFFB3,
    RES180 = 0xFFB4,
    RES181 = 0xFFB5,
    RES182 = 0xFFB6,
    RES183 = 0xFFB7,
    RES184 = 0xFFB8,
    RES185 = 0xFFB9,
    RES186 = 0xFFBA,
    RES187 = 0xFFBB,
    RES188 = 0xFFBC,
    RES189 = 0xFFBD,
    RES190 = 0xFFBE,
    RES191 = 0xFFBF,
}

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

    /// Some markers stand alone, that is, which is not the start of a markery segment.
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

    pub fn encoding_process(&self) -> (CodingProcess, EntropyCoding) {
        match self {
            Marker::SOF0 => (CodingProcess::BaselineDCT, EntropyCoding::Huffman(vec![])),
            Marker::SOF1 => (
                CodingProcess::ExtendedSequentialDCT,
                EntropyCoding::Huffman(vec![]),
            ),
            _ => unreachable!(),
        }
    }

    pub fn to_u16(&self) -> u16 {
        u16::from_be_bytes([Marker::GLOBAL as u8, *self as u8])
    }
}
