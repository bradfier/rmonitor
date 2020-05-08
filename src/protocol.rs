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
}

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("unknown record type {}", .0)]
    UnknownRecordType(String),
    #[error("malformed record")]
    MalformedRecord,
    #[error("unknown flag state '{}'", .0)]
    UnknownFlagState(String),
    #[error("invalid integer field")]
    InvalidIntegerField(#[from] ParseIntError),
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

trait RMonitorField<T> {
    fn decode(self) -> Result<T, RecordError>;
}

impl RMonitorField<String> for &str {
    fn decode(self) -> Result<String, RecordError> {
        Ok(self.trim_matches('"').to_owned())
    }
}

impl RMonitorField<Flag> for &str {
    fn decode(self) -> Result<Flag, RecordError> {
        Ok(self.trim_matches('"').parse()?)
    }
}

impl RMonitorField<u32> for &str {
    fn decode(self) -> Result<u32, RecordError> {
        Ok(self.parse()?)
    }
}

impl RMonitorField<Option<u32>> for &str {
    fn decode(self) -> Result<Option<u32>, RecordError> {
        if self.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(self.parse()?))
        }
    }
}

impl RMonitorField<u16> for &str {
    fn decode(self) -> Result<u16, RecordError> {
        Ok(self.parse()?)
    }
}

impl RMonitorField<u8> for &str {
    fn decode(self) -> Result<u8, RecordError> {
        Ok(self.parse()?)
    }
}

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
}

impl Record {
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
            _ => Err(RecordError::UnknownRecordType(splits[0].to_owned())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Heartbeat {
    pub laps_to_go: u32,
    pub time_to_go: String,
    pub time_of_day: String,
    pub race_time: String,
    pub flag_status: Flag,
}

impl Heartbeat {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 6 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            laps_to_go: parts[1].decode()?,
            time_to_go: parts[2].decode()?,
            time_of_day: parts[3].decode()?,
            race_time: parts[4].decode()?,
            flag_status: parts[5].decode()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Competitor {
    registration_number: String,
    number: String,
    transponder_number: u32,
    first_name: String,
    last_name: String,
    nationality: String,
    class_number: u8,
}

impl Competitor {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 8 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            registration_number: parts[1].decode()?,
            number: parts[2].decode()?,
            transponder_number: parts[3].decode()?,
            first_name: parts[4].decode()?,
            last_name: parts[5].decode()?,
            nationality: parts[6].decode()?,
            class_number: parts[7].decode()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CompetitorExt {
    registration_number: String,
    number: String,
    class_number: u8,
    first_name: String,
    last_name: String,
    nationality: String,
    additional_data: String,
}

impl CompetitorExt {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 8 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            registration_number: parts[1].decode()?,
            number: parts[2].decode()?,
            class_number: parts[3].decode()?,
            first_name: parts[4].decode()?,
            last_name: parts[5].decode()?,
            nationality: parts[6].decode()?,
            additional_data: parts[7].decode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Run {
    number: u8,
    description: String,
}

impl Run {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 3 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            number: parts[1].decode()?,
            description: parts[2].decode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    number: u8,
    description: String,
}

impl Class {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 3 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            number: parts[1].decode()?,
            description: parts[2].decode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Setting {
    description: String,
    value: String,
}

impl Setting {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 3 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            description: parts[1].decode()?,
            value: parts[2].decode()?,
        })
    }
}

// Race _position_ information, this is referred to as a 'Race information' field
// in the protocol specification
#[derive(Debug, Clone)]
pub struct Race {
    position: u16,
    registration_number: String,
    laps: Option<u32>,
    total_time: String,
}

impl Race {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 5 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            position: parts[1].decode()?,
            registration_number: parts[2].decode()?,
            laps: parts[3].decode()?,
            total_time: parts[4].decode()?,
        })
    }
}

// Practice / Qualifying position information (best lap etc)
#[derive(Debug, Clone)]
pub struct PracticeQual {
    position: u16,
    registration_number: String,
    best_lap: u32,
    best_laptime: String,
}

impl PracticeQual {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 5 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            position: parts[1].decode()?,
            registration_number: parts[2].decode()?,
            best_lap: parts[3].decode()?,
            best_laptime: parts[4].decode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Init {
    time: String,
    date: String,
}

impl Init {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 3 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            time: parts[1].decode()?,
            date: parts[2].decode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Passing {
    registration_number: String,
    laptime: String,
    total_time: String,
}

impl Passing {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 4 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            registration_number: parts[1].decode()?,
            laptime: parts[2].decode()?,
            total_time: parts[3].decode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Correction {
    registration_number: String,
    number: String,
    laps: u32,
    total_time: String,
    correction: String,
}

impl Correction {
    pub fn decode(parts: &[&str]) -> Result<Self, RecordError> {
        if parts.len() != 6 {
            return Err(RecordError::MalformedRecord);
        }

        Ok(Self {
            registration_number: parts[1].decode()?,
            number: parts[2].decode()?,
            laps: parts[3].decode()?,
            total_time: parts[4].decode()?,
            correction: parts[5].decode()?,
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
}
