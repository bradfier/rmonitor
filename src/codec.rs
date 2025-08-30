//! A tokio-util [`Decoder`] implementation for RMonitor.
//!
//! [`Decoder`]: tokio_util::codec::Decoder
use thiserror::Error;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, LinesCodec, LinesCodecError};

use crate::protocol::*;

/// An error was encountered when trying to decode an RMonitor record from
/// the input byte stream.
#[derive(Error, Debug)]
pub enum RMonitorCodecError {
    /// An error occured when trying to decode a Record from an otherwise valid line
    #[error("unable to decode record from line '{line}': {source}")]
    RecordDecode {
        line: String,
        #[source]
        source: RecordError,
    },
    /// The underlying LinesCodec encountered an error trying to extract a single line
    #[error(transparent)]
    LinesCodec(#[from] LinesCodecError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A decoder for RMonitor records, which wraps an underlying [`LinesCodec`]
/// to provide framing logic.
///
/// [`LinesCodec`]: tokio_util::codec::LinesCodec
#[derive(Default, Debug)]
pub struct RMonitorDecoder {
    lines_codec: LinesCodec,
}

impl RMonitorDecoder {
    /// Returns an `RMonitorDecoder` for decoding RMonitor records from a TCP stream.
    ///
    /// # Note
    ///
    /// The returned `RMonitorDecoder` will have an underlying `LinesCodec` with no upper
    /// bound on the length of a buffered line. Consider using [`new_with_max_length`] instead.
    ///
    /// [`new_with_max_length`]: crate::codec::RMonitorDecoder::new_with_max_length()
    pub fn new() -> Self {
        Self {
            lines_codec: LinesCodec::new(),
        }
    }

    /// Returns an `RMonitorDecoder` where the underlying `LinesCodec` has a maximum line length
    /// limit.
    ///
    /// It is recommended to set such a limit where the input to be supplied to the decoder is
    /// untrusted, as an attacker could send an unbounded amount of input with no newline
    /// characters.
    pub fn new_with_max_length(max_length: usize) -> Self {
        Self {
            lines_codec: LinesCodec::new_with_max_length(max_length),
        }
    }
}

impl Decoder for RMonitorDecoder {
    type Item = Record;
    type Error = RMonitorCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let line = self.lines_codec.decode(src)?;
        if let Some(line) = line {
            // If we've somehow started decoding in the middle of a record, or this line is
            // completely empty, discard it and continue from the next one.
            if line.is_empty() || line.as_bytes()[0] != b'$' {
                return Ok(None);
            }

            // Try to decode the record, but if it fails, log the error and skip this line
            match Record::decode(&line) {
                Ok(record) => Ok(Some(record)),
                Err(source) => {
                    // Log the error with full details but continue processing
                    log::warn!(
                        "Skipping invalid RMonitor record from line '{}': {}",
                        line,
                        source
                    );
                    // Return Ok(None) to continue processing the next line
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consume(
        decoder: &mut RMonitorDecoder,
        bytes: &mut BytesMut,
    ) -> Vec<Result<Option<Record>, RMonitorCodecError>> {
        let mut result = vec![];
        let mut consecutive_none = 0;
        loop {
            match decoder.decode(bytes) {
                Ok(None) => {
                    consecutive_none += 1;
                    // If we get two consecutive None results, assume we're done
                    // (one for skipped record, one for no more complete lines)
                    if consecutive_none > 1 || bytes.is_empty() {
                        break;
                    }
                }
                Ok(Some(record)) => {
                    consecutive_none = 0;
                    result.push(Ok(Some(record)));
                }
                Err(e) => {
                    result.push(Err(e));
                    break;
                }
            }
        }
        result
    }

    #[test]
    fn test_decodes_single_line() {
        let mut decoder = RMonitorDecoder::new_with_max_length(2048);
        let mut bytes =
            BytesMut::from("$F,9999,\"00:00:00\",\"14:09:52\",\"00:59:59\",\"      \"\r\n");

        let result = consume(&mut decoder, &mut bytes);

        assert_eq!(0, bytes.len());
        assert_eq!(1, result.len());

        assert!(matches!(result[0], Ok(Some(Record::Heartbeat(_)))));
    }

    #[test]
    fn test_decodes_large_sample() {
        let mut decoder = RMonitorDecoder::new_with_max_length(2048);
        let data: Vec<u8> = std::fs::read("sample/2009_Sebring_ALMS_Session_5.txt").unwrap();

        let mut bytes = BytesMut::from(data.as_slice());

        let result = consume(&mut decoder, &mut bytes);

        // All bytes were consumed
        assert_eq!(0, bytes.len());
        // And no errors were encountered when processing the file
        assert!(result.into_iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_skips_invalid_records() {
        // Initialize logging for tests
        let _ = env_logger::builder().is_test(true).try_init();

        let mut decoder = RMonitorDecoder::new_with_max_length(2048);
        // Create a mix of valid and invalid records
        let mut bytes = BytesMut::from(
            "$F,invalid,data,here\r\n$F,9999,\"00:00:00\",\"14:09:52\",\"00:59:59\",\"      \"\r\n$UNKNOWN,some,bad,record\r\n$B,5,\"Friday free practice\"\r\n"
        );

        let result = consume(&mut decoder, &mut bytes);

        // Debug: Print how many bytes remain
        println!("Remaining bytes: {}", bytes.len());
        println!("Results: {:?}", result);

        // All bytes should be consumed
        assert_eq!(0, bytes.len());

        // We should get 2 valid records (the valid heartbeat and run record)
        // The invalid records should be skipped (returning Ok(None))
        let valid_records: Vec<_> = result
            .into_iter()
            .filter_map(|r| r.ok().flatten())
            .collect();

        assert_eq!(2, valid_records.len());
        assert!(matches!(valid_records[0], Record::Heartbeat(_)));
        assert!(matches!(valid_records[1], Record::Run(_)));
    }
}
