use bytes::BytesMut;
use thiserror::Error;
use tokio_codec::{Decoder, LinesCodec, LinesCodecError};

use crate::protocol::*;

#[derive(Error, Debug)]
pub enum RMonitorCodecError {
    #[error("unable to decode record from line")]
    RecordDecode(#[from] RecordError),
    // The underlying lines codec encountered an error
    #[error(transparent)]
    LinesCodec(#[from] LinesCodecError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct RMonitorDecoder {
    lines_codec: LinesCodec,
}

impl RMonitorDecoder {
    pub fn new(max_length: usize) -> Self {
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
            // If we've somehow started decoding in the middle of a record, discard
            // this line and continue from the next one.
            if line.as_bytes()[0] != b'$' {
                return Ok(None);
            }
            Ok(Some(Record::decode(&line)?))
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
        loop {
            match decoder.decode(bytes) {
                Ok(None) => {
                    break;
                }
                out => result.push(out),
            }
        }
        result
    }

    #[test]
    fn test_decodes_single_line() {
        let mut decoder = RMonitorDecoder::new(2048);
        let mut bytes = BytesMut::from(
            b"$F,9999,\"00:00:00\",\"14:09:52\",\"00:59:59\",\"      \"\r\n".to_vec(),
        );

        let result = consume(&mut decoder, &mut bytes);

        assert_eq!(0, bytes.len());
        assert_eq!(1, result.len());

        assert!(matches!(result[0], Ok(Some(Record::Heartbeat(_)))));
    }

    #[test]
    fn test_decodes_large_sample() {
        let mut decoder = RMonitorDecoder::new(2048);
        let data: Vec<u8> = std::fs::read("sample/2009_Sebring_ALMS_Session_5.txt").unwrap();

        let mut bytes = BytesMut::from(data);

        let result = consume(&mut decoder, &mut bytes);

        // All bytes were consumed
        assert_eq!(0, bytes.len());
        // And no errors were encountered when processing the file
        assert!(result.into_iter().all(|r| r.is_ok()));
    }
}
