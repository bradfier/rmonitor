//! Protocol implementation for decoding RMonitor messages.
//!
//! # Example
//!
//! ```
//! use rmonitor::protocol::Record;
//!
//! let data = r#"$A,"1234BE","12X",52474,"John","Johnson","USA",5"#;
//! let record = Record::decode(&data);
//!
//! assert!(record.is_ok());
//! assert!(matches!(record, Ok(Record::Competitor(_))));
//!
//! if let Ok(Record::Competitor(competitor)) = record {
//!     assert_eq!(competitor.registration_number, "1234BE");
//!     assert_eq!(competitor.number, "12X");
//!     assert_eq!(competitor.transponder_number, 52474);
//!     assert_eq!(competitor.first_name, "John");
//!     assert_eq!(competitor.last_name, "Johnson");
//!     assert_eq!(competitor.nationality, "USA");
//!     assert_eq!(competitor.class_number, 5);
//! }
//! ```

use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

/// RMonitor commands are represented in messages by ASCII strings
pub mod command {
    pub const HEARTBEAT: &str = "$F";
    pub const COMPETITOR: &str = "$A";
    pub const COMPETITOR_EXT: &str = "$COMP";
    pub const RUN: &str = "$B";
    pub const CLASS: &str = "$C";
    pub const SETTING: &str = "$E";
    pub const RACE: &str = "$G";
    pub const PRAC_QUAL: &str = "$H";
    pub const INIT: &str = "$I";
    pub const PASSING: &str = "$J";
    pub const CORRECTION: &str = "$COR";

    // IMSA enhanced protocol messages
    pub const LINE_CROSSING: &str = "$L";
    pub const TRACK_DESCRIPTION: &str = "$T";
}

/// An error occured when decoding a record
#[derive(Error, Debug)]
pub enum RecordError {
    /// The record prefix was not recognised as a valid record type
    #[error("unknown record type {}", .0)]
    UnknownRecordType(String),
    /// The input could not be decoded as the record type indicated by the prefix
    #[error("malformed record")]
    MalformedRecord,
    /// A heartbeat record included an unrecognised flag state
    #[error("unknown flag state '{}'", .0)]
    UnknownFlagState(String),
    /// A numeric record field could't be parsed as an integer
    #[error("invalid integer field")]
    InvalidIntegerField(#[from] ParseIntError),
    /// An IMSA track description record had a different number of sections than specified
    #[error("track description had different number of sections than specified")]
    IncorrectSectionCount,
}

#[derive(Copy, Clone, Debug)]
pub enum Flag {
    None,
    Green,
    Yellow,
    Red,
    Finish,
}

impl FromStr for Flag {
    type Err = RecordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Flag states are fixed width, with trailing spaces
        match s {
            "      " => Ok(Flag::None),
            "Green " => Ok(Flag::Green),
            "Yellow" => Ok(Flag::Yellow),
            "Red   " => Ok(Flag::Red),
            "Finish" => Ok(Flag::Finish),
            _ => Err(RecordError::UnknownFlagState(s.to_owned())),
        }
    }
}

/// Implemented for types which can be constructed from the comma-separated parts of an RMonitor
/// line.
trait FromParts: Sized {
    fn decode(parts: &[&str]) -> Result<Self, RecordError>;
}

macro_rules! decode_impl {
    ($type:ident, $count:expr, $($field:ident),+) => (
        impl FromParts for $type {
            fn decode(parts: &[&str]) -> Result<Self, RecordError> {
                if parts.len() != $count {
                    return Err(RecordError::MalformedRecord);
                }

                // A little clunky, but should optimize out
                let mut idx = 0;
                $(
                    idx += 1;
                    let $field = parts[idx].decode()?;
                )*

                Ok(Self {
                    $(
                        $field,
                    )*
                })
            }
        }
    )
}

/// Implemented for types which can be constructed from a single RMonitor message part.
trait FieldExt<T> {
    fn decode(self) -> Result<T, RecordError>;
}

impl FieldExt<String> for &str {
    fn decode(self) -> Result<String, RecordError> {
        Ok(self.trim_matches('"').to_owned())
    }
}

impl FieldExt<Flag> for &str {
    fn decode(self) -> Result<Flag, RecordError> {
        Ok(self.trim_matches('"').parse()?)
    }
}

impl FieldExt<u32> for &str {
    fn decode(self) -> Result<u32, RecordError> {
        Ok(self.parse()?)
    }
}

impl FieldExt<Option<u32>> for &str {
    fn decode(self) -> Result<Option<u32>, RecordError> {
        if self.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.parse()?))
        }
    }
}

impl FieldExt<u16> for &str {
    fn decode(self) -> Result<u16, RecordError> {
        Ok(self.parse()?)
    }
}

impl FieldExt<u8> for &str {
    fn decode(self) -> Result<u8, RecordError> {
        Ok(self.parse()?)
    }
}

/// A unit of data from the RMonitor protocol
#[derive(Clone, Debug)]
pub enum Record {
    Heartbeat(Heartbeat),
    Competitor(Competitor),
    CompetitorExt(CompetitorExt),
    Run(Run),
    Class(Class),
    Setting(Setting),
    Race(Race),
    PracticeQual(PracticeQual),
    Init(Init),
    Passing(Passing),
    Correction(Correction),
    LineCrossing(LineCrossing),
    TrackDescription(TrackDescription),
}

impl Record {
    /// Decodes a record from a single line of valid UTF-8 text
    pub fn decode(line: &str) -> Result<Self, RecordError> {
        let splits: Vec<&str> = line.split(',').collect();

        if splits.len() < 2 {
            return Err(RecordError::MalformedRecord);
        }

        match splits[0] {
            command::HEARTBEAT => Ok(Record::Heartbeat(Heartbeat::decode(&splits)?)),
            command::COMPETITOR => Ok(Record::Competitor(Competitor::decode(&splits)?)),
            command::COMPETITOR_EXT => Ok(Record::CompetitorExt(CompetitorExt::decode(&splits)?)),
            command::RUN => Ok(Record::Run(Run::decode(&splits)?)),
            command::CLASS => Ok(Record::Class(Class::decode(&splits)?)),
            command::SETTING => Ok(Record::Setting(Setting::decode(&splits)?)),
            command::RACE => Ok(Record::Race(Race::decode(&splits)?)),
            command::PRAC_QUAL => Ok(Record::PracticeQual(PracticeQual::decode(&splits)?)),
            command::INIT => Ok(Record::Init(Init::decode(&splits)?)),
            command::PASSING => Ok(Record::Passing(Passing::decode(&splits)?)),
            command::CORRECTION => Ok(Record::Correction(Correction::decode(&splits)?)),
            command::LINE_CROSSING => Ok(Record::LineCrossing(LineCrossing::decode(&splits)?)),
            command::TRACK_DESCRIPTION => {
                Ok(Record::TrackDescription(TrackDescription::decode(&splits)?))
            }
            _ => Err(RecordError::UnknownRecordType(splits[0].to_owned())),
        }
    }
}

/// Heartbeat message, sent every second that a session is active
#[derive(Clone, Debug)]
pub struct Heartbeat {
    /// Number of laps to go
    pub laps_to_go: u32,
    /// Time until the session ends
    pub time_to_go: String,
    /// The current time (usually in UTC, but dependent on the timing system in use)
    pub time_of_day: String,
    /// The time from the first green flag
    pub race_time: String,
    /// Current flag status
    pub flag_status: Flag,
}

decode_impl!(
    Heartbeat,
    6,
    laps_to_go,
    time_to_go,
    time_of_day,
    race_time,
    flag_status
);

/// Competitor information record
///
/// Competitors are unqiuely keyed on their `registration_number` field.
#[derive(Clone, Debug)]
pub struct Competitor {
    pub registration_number: String,
    pub number: String,
    pub transponder_number: u32,
    pub first_name: String,
    pub last_name: String,
    /// Often used for Make/Model or Team name by some timing software
    pub nationality: String,
    /// Unique class number (matches a `Class` record)
    pub class_number: u8,
}

decode_impl!(
    Competitor,
    8,
    registration_number,
    number,
    transponder_number,
    first_name,
    last_name,
    nationality,
    class_number
);

/// Extended competitor information
///
/// It's unclear why the protocol includes this extra (almost identical) competitor information
/// message, but it is included for completeness.
#[derive(Clone, Debug)]
pub struct CompetitorExt {
    pub registration_number: String,
    pub number: String,
    pub class_number: u8,
    pub first_name: String,
    pub last_name: String,
    pub nationality: String,
    pub additional_data: String,
}

decode_impl!(
    CompetitorExt,
    8,
    registration_number,
    number,
    class_number,
    first_name,
    last_name,
    nationality,
    additional_data
);

/// Run (session) information
#[derive(Debug, Clone)]
pub struct Run {
    /// Defined as 'unique', it's likely this means unique within a single RMonitor session
    pub number: u8,
    pub description: String,
}

decode_impl!(Run, 3, number, description);

/// Class information
#[derive(Debug, Clone)]
pub struct Class {
    /// Defined as 'unique', it's likely this means unique within a single RMonitor session
    pub number: u8,
    pub description: String,
}

decode_impl!(Class, 3, number, description);

/// Track setting information
///
/// This message type supports arbitrary key-value pairs, however the only specified keys in the
/// protocol documentation are:
///
/// - 'TRACKNAME': The name of the track / event venue
/// - 'TRACKLENGTH': The length of the track / event venue
#[derive(Debug, Clone)]
pub struct Setting {
    pub description: String,
    /// Specified as a `String` for both defined keys, however `TRACKLENGTH` is normally a string
    /// representation of a decimal number (e.g. '2.500')
    pub value: String,
}

decode_impl!(Setting, 3, description, value);

/// Race position information
///
/// Contains the current race position of a competitor (identified by `registration_number`) and
/// their total time / laps completed.
///
/// # Note
///
/// If an overtake occurs, the timing software should emit a `Race` record for both the passing and
/// passed competitors, updated with their new positions, so you shouldn't need to recompute the
/// running order yourself.
///
/// Both `Race` and `PracticeQual` messages should be expected in all types of session, in all
/// scenarios they provide information about the competitor's best lap and total race time, the
/// interpretation of the standings will depend on the type of session in progress.
#[derive(Debug, Clone)]
pub struct Race {
    /// The competitor's position in the running order
    pub position: u16,
    pub registration_number: String,
    /// Laps completed, this will be `None` if the competitor has not yet completed a lap after a
    /// Green flag state has occured.
    pub laps: Option<u32>,
    /// Total race time (the sentinel value `00:59:59.999` indicates a competitor for whom no
    /// passing has yet been recorded).
    pub total_time: String,
}

decode_impl!(Race, 5, position, registration_number, laps, total_time);

/// Practice / Qualification position information
///
/// Contains the current position of a competitor in the standings for a Practice or Qualifying
/// session.
///
/// # Note
///
/// As with a `Race` record, the timing software should issue multiple `PracticeQual` messages when
/// the standings change.
#[derive(Debug, Clone)]
pub struct PracticeQual {
    /// The competitor's position in the fastest-lap standings
    pub position: u16,
    pub registration_number: String,
    /// The lap number of the best lap
    pub best_lap: u32,
    /// The laptime of the best lap
    pub best_laptime: String,
}

decode_impl!(
    PracticeQual,
    5,
    position,
    registration_number,
    best_lap,
    best_laptime
);

/// Indicates that the scoreboard should be reset
///
/// The timing software may send an `Init` message immediately prior to the start of a new session,
/// or when it has determined the data is stale and should be completely refreshed.
#[derive(Debug, Clone)]
pub struct Init {
    pub time: String,
    pub date: String,
}

decode_impl!(Init, 3, time, date);

/// Passing information
///
/// Sent each time a competitor crosses the main timeline.
#[derive(Debug, Clone)]
pub struct Passing {
    pub registration_number: String,
    pub laptime: String,
    pub total_time: String,
}

decode_impl!(Passing, 4, registration_number, laptime, total_time);

/// Corrected finish time
///
/// Sent each time a passing time is corrected (this can be due to a photocell time being
/// associated with a competitor after the `Passing` message was already sent).
#[derive(Debug, Clone)]
pub struct Correction {
    pub registration_number: String,
    pub number: String,
    pub laps: u32,
    /// The corrected total time
    pub total_time: String,
    /// The total time corrections from the previous passing message
    pub correction: String,
}

decode_impl!(
    Correction,
    6,
    registration_number,
    number,
    laps,
    total_time,
    correction
);

/// Timeline crossing message
///
/// Sent each time a competitor crosses a timeline, this message type is part of the IMSA Enhanced
/// specification.
#[derive(Debug, Clone)]
pub struct LineCrossing {
    pub number: String,
    pub timeline_number: String,
    pub timeline_name: String,
    pub date: String,
    pub time: String,
    // The following fields are referenced in the IMSA protocol document
    // but don't appear in any of the sample data.
    pub driver_id: Option<u8>,
    pub class_name: Option<String>,
}

// Manual implementation to support the variadic fields
impl FromParts for LineCrossing {
    fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() < 6 {
            return Err(RecordError::MalformedRecord);
        }

        let driver_id = parts
            .get(6)
            .map(|p| p.decode())
            .map_or(Ok(None), |r| r.map(Some))?;

        let class_name = parts
            .get(7)
            .map(|p| p.decode())
            .map_or(Ok(None), |r| r.map(Some))?;

        Ok(Self {
            number: parts[1].decode()?,
            timeline_number: parts[2].decode()?,
            timeline_name: parts[3].decode()?,
            date: parts[4].decode()?,
            time: parts[5].decode()?,
            driver_id,
            class_name,
        })
    }
}

/// Track and timeline description message
///
/// Contains track information as well as a variable number of [`TrackSection`]s, which define the
/// distance between two timelines.
///
/// This message type is part of the IMSA Enhanced specification.
///
/// [`TrackSection`]: crate::protocol::TrackSection
#[derive(Debug, Clone)]
pub struct TrackDescription {
    pub name: String,
    pub short_name: String,
    pub distance: String,
    pub sections: Vec<TrackSection>,
}

/// Track section field
///
/// Describes a section of track between two timelines.
#[derive(Debug, Clone)]
pub struct TrackSection {
    /// Section name
    pub name: String,
    /// Timeline number at section start
    pub start: String,
    /// Timeline number at section end
    pub end: String,
    /// Sector distance, given in whole inches
    pub distance: u32,
}

impl FromParts for TrackDescription {
    fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() < 5 {
            return Err(RecordError::MalformedRecord);
        }

        let expected: usize = parts[4].parse()?;
        let sections: Vec<TrackSection> = parts[5..]
            .chunks(4)
            .filter(|s| s.len() == 4) // Discard short sections
            .map(|s| {
                Ok(TrackSection {
                    name: s[0].decode()?,
                    start: s[1].decode()?,
                    end: s[2].decode()?,
                    distance: s[3].decode()?,
                })
            })
            .collect::<Result<Vec<TrackSection>, RecordError>>()?;

        if sections.len() != expected {
            return Err(RecordError::IncorrectSectionCount);
        }

        Ok(Self {
            name: parts[1].decode()?,
            short_name: parts[2].decode()?,
            distance: parts[3].decode()?,
            sections,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decodes_unknown_record() {
        let data = "$ZZZ,5,\"Friday free practice\"";
        let record = Record::decode(&data);

        assert!(record.is_err());
        assert!(matches!(record, Err(RecordError::UnknownRecordType(_))));
    }

    #[test]
    fn test_decodes_heartbeat() {
        let data = "$F,14,\"00:12:45\",\"13:34:23\",\"00:09:47\",\"Green \"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Heartbeat(Heartbeat { laps_to_go: 14, .. }))));
    }

    #[test]
    fn test_decodes_competitor() {
        let data = "$A,\"1234BE\",\"12X\",52474,\"John\",\"Johnson\",\"USA\",5";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Competitor(_))));

        if let Ok(Record::Competitor(competitor)) = record {
            assert_eq!(competitor.registration_number, "1234BE");
            assert_eq!(competitor.number, "12X");
            assert_eq!(competitor.transponder_number, 52474);
            assert_eq!(competitor.first_name, "John");
            assert_eq!(competitor.last_name, "Johnson");
            assert_eq!(competitor.nationality, "USA");
            assert_eq!(competitor.class_number, 5);
        }
    }

    #[test]
    fn test_decodes_competitor_ext() {
        let data = "$COMP,\"1234BE\",\"12X\",5,\"John\",\"Johnson\",\"USA\",\"CAMEL\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::CompetitorExt(_))));

        if let Ok(Record::CompetitorExt(competitor)) = record {
            assert_eq!(competitor.registration_number, "1234BE");
            assert_eq!(competitor.number, "12X");
            assert_eq!(competitor.first_name, "John");
            assert_eq!(competitor.last_name, "Johnson");
            assert_eq!(competitor.nationality, "USA");
            assert_eq!(competitor.additional_data, "CAMEL");
            assert_eq!(competitor.class_number, 5);
        }
    }

    #[test]
    fn test_decodes_run() {
        let data = "$B,5,\"Friday free practice\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Run(_))));

        if let Ok(Record::Run(run)) = record {
            assert_eq!(run.number, 5);
            assert_eq!(run.description, "Friday free practice");
        }
    }

    #[test]
    fn test_decodes_class() {
        let data = "$C,5,\"Formula 3000\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Class(_))));

        if let Ok(Record::Class(class)) = record {
            assert_eq!(class.number, 5);
            assert_eq!(class.description, "Formula 3000");
        }
    }

    #[test]
    fn test_decodes_settings() {
        // Two samples provided for this protocol record
        let data = "$E,\"TRACKNAME\",\"Indianapolis Motor Speedway\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Setting(_))));

        if let Ok(Record::Setting(setting)) = record {
            assert_eq!(setting.description, "TRACKNAME");
            assert_eq!(setting.value, "Indianapolis Motor Speedway");
        }

        let data = "$E,\"TRACKLENGTH\",\"2.500\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Setting(_))));

        if let Ok(Record::Setting(setting)) = record {
            assert_eq!(setting.description, "TRACKLENGTH");
            assert_eq!(setting.value, "2.500");
        }
    }

    #[test]
    fn test_decodes_race() {
        let data = "$G,3,\"1234BE\",14,\"01:12:47.872\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Race(_))));

        if let Ok(Record::Race(race)) = record {
            assert_eq!(race.position, 3);
            assert_eq!(race.registration_number, "1234BE");
            assert_eq!(race.laps, Some(14));
            assert_eq!(race.total_time, "01:12:47.872");
        }
    }

    #[test]
    fn test_decodes_practice_qual() {
        let data = "$H,2,\"1234BE\",3,\"00:02:17.872\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::PracticeQual(_))));

        if let Ok(Record::PracticeQual(pq)) = record {
            assert_eq!(pq.position, 2);
            assert_eq!(pq.registration_number, "1234BE");
            assert_eq!(pq.best_lap, 3);
            assert_eq!(pq.best_laptime, "00:02:17.872");
        }
    }

    #[test]
    fn test_decodes_init_command() {
        let data = "$I,\"16:36:08.000\",\"12 jan 01\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Init(_))));

        if let Ok(Record::Init(init)) = record {
            assert_eq!(init.time, "16:36:08.000");
            assert_eq!(init.date, "12 jan 01");
        }
    }

    #[test]
    fn test_decodes_passing() {
        let data = "$J,\"1234BE\",\"00:02:03.826\",\"01:42:17.672\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Passing(_))));

        if let Ok(Record::Passing(passing)) = record {
            assert_eq!(passing.registration_number, "1234BE");
            assert_eq!(passing.laptime, "00:02:03.826");
            assert_eq!(passing.total_time, "01:42:17.672");
        }
    }

    #[test]
    fn test_decodes_correction() {
        let data = "$COR,\"123BE\",\"658\",2,\"00:00:35.272\",\"+00:00:00.012\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::Correction(_))));

        if let Ok(Record::Correction(cor)) = record {
            assert_eq!(cor.registration_number, "123BE");
            assert_eq!(cor.number, "658");
            assert_eq!(cor.laps, 2);
            assert_eq!(cor.correction, "+00:00:00.012");
        }
    }

    #[test]
    fn test_decodes_line_crossing() {
        // Fields seen in protocol spec
        let data = "$L,\"13\",\"P2\",\"POP\",\"01/27/2009\",\"10:10:20.589\",1,\"PC\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::LineCrossing(_))));

        if let Ok(Record::LineCrossing(c)) = record {
            assert_eq!(c.number, "13");
            assert_eq!(c.timeline_number, "P2");
            assert_eq!(c.timeline_name, "POP");
            assert_eq!(c.date, "01/27/2009");
            assert_eq!(c.time, "10:10:20.589");
            assert_eq!(c.driver_id, Some(1));
            assert_eq!(c.class_name, Some("PC".to_owned()));
        }

        // Fields seen in sample data
        let data = "$L,\"15\",\"P1\",\"SFP\",\"01/27/2009\",\"14:13:22.818\"";
        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::LineCrossing(_))));

        if let Ok(Record::LineCrossing(c)) = record {
            assert_eq!(c.number, "15");
            assert_eq!(c.timeline_number, "P1");
            assert_eq!(c.timeline_name, "SFP");
            assert_eq!(c.date, "01/27/2009");
            assert_eq!(c.time, "14:13:22.818");
            assert_eq!(c.driver_id, None);
            assert_eq!(c.class_name, None);
        }
    }

    #[test]
    fn test_decodes_track_description() {
        let data = concat!(
            r#"$T,"Circuit of the Americas","COTA","3.40",15,"#,
            r#""S01","T1","T2",3375,"S02","T2","T3",36559,"S03","T3","T4",40933,"S04","T4","T5",13256,"S05","T5",""#,
            r#"T6",20923,"S06","T6","T7",1181,"S07","T7","T8",12711,"S08","T8","T9",1181,"S09","T9","TA",29313,"S1"#,
            r#"0","TA","TB",41744,"S11","TB","T1",16113,"LAP","T1","P1",217379,"PIT","PB","P2",19688,"SP4","T6","T"#,
            r#"7",1181,"SP5","T8","T9",1181"#
        );

        let record = Record::decode(&data);

        assert!(record.is_ok());
        assert!(matches!(record, Ok(Record::TrackDescription(_))));

        if let Ok(Record::TrackDescription(td)) = record {
            assert_eq!(td.name, "Circuit of the Americas");
            assert_eq!(td.short_name, "COTA");
            assert_eq!(td.distance, "3.40");
            assert_eq!(td.sections.len(), 15);
        }
    }

    #[test]
    fn test_errors_wrong_track_section_count() {
        let data = concat!(
            r#"$T,"Circuit of the Americas","COTA","3.40",15,"#,
            r#""S01","T1","T2",3375,"S02","T2","T3",36559,"S03","T3","T4",40933,"S04","T4","T5",13256,"S05","T5",""#,
            r#"T6",20923,"S06","T6","T7",1181,"S07","T7","T8",12711,"S08","T8","T9",1181,"S09","T9","TA",29313,"S1"#,
            r#"0","TA","TB",41744,"S11","TB","T1",16113,"LAP","T1","P1",217379,"PIT","PB","P2",19688"#
        );

        let record = Record::decode(&data);
        assert!(record.is_err());
        assert!(matches!(record, Err(RecordError::IncorrectSectionCount)))
    }
}
