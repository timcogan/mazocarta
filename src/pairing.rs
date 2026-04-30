use std::io::{Read, Write};

use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use serde::{Deserialize, Serialize};

const BASE45_ALPHABET: &[u8; 45] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
const PAIR_PREFIX_OFFER_V1: &str = "MZO1";
const PAIR_PREFIX_ANSWER_V1: &str = "MZA1";
const PAIR_TRANSPORT_PREFIX: &str = "MZQ1";
const PAIR_CODEC_PLAIN: char = 'P';
const PAIR_CODEC_COMPRESSED: char = 'Z';

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairPayloadInput {
    kind: String,
    invitation_id: String,
    description: PairDescription,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PairPayloadOutput {
    kind: String,
    version: u8,
    invitation_id: String,
    description: PairDescription,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PairDescription {
    #[serde(rename = "type")]
    description_type: String,
    sdp: String,
}

#[derive(Clone, Debug)]
struct TransportFrame {
    invitation_id: String,
    kind: char,
    index: usize,
    count: usize,
    chunk: String,
}

#[derive(Clone, Debug)]
struct TransportAssembly {
    invitation_id: String,
    kind: char,
    count: usize,
    parts: Vec<Option<String>>,
}

#[derive(Clone, Debug)]
pub(crate) enum TransportSubmitResult {
    DirectCode,
    Partial,
    Complete,
}

#[derive(Default)]
pub(crate) struct PairingBridge {
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
    decoded_payload_buffer: Vec<u8>,
    frames: Vec<String>,
    assembly: Option<TransportAssembly>,
    received_parts: usize,
    total_parts: usize,
}

impl PairingBridge {
    pub(crate) fn prepare_input_buffer(&mut self, len: usize) -> *mut u8 {
        self.input_buffer.resize(len, 0);
        self.input_buffer.as_mut_ptr()
    }

    fn read_input_string(&self, len: usize) -> Result<String, String> {
        if len > self.input_buffer.len() {
            return Err("Pairing input buffer overflow.".to_string());
        }
        std::str::from_utf8(&self.input_buffer[..len])
            .map(str::to_owned)
            .map_err(|_| "Pairing input was not valid UTF-8.".to_string())
    }

    fn set_output_string(&mut self, raw: &str) {
        self.output_buffer.clear();
        self.output_buffer.extend_from_slice(raw.as_bytes());
    }

    fn set_decoded_payload_json(&mut self, raw: &str) {
        self.decoded_payload_buffer.clear();
        self.decoded_payload_buffer
            .extend_from_slice(raw.as_bytes());
    }

    pub(crate) fn output_ptr(&self) -> *const u8 {
        self.output_buffer.as_ptr()
    }

    pub(crate) fn output_len(&self) -> usize {
        self.output_buffer.len()
    }

    pub(crate) fn decoded_payload_ptr(&self) -> *const u8 {
        self.decoded_payload_buffer.as_ptr()
    }

    pub(crate) fn decoded_payload_len(&self) -> usize {
        self.decoded_payload_buffer.len()
    }

    pub(crate) fn encode_payload_from_buffer(&mut self, len: usize) -> Result<(), String> {
        let raw = self.read_input_string(len)?;
        let encoded = encode_pair_payload_from_json(&raw)?;
        self.set_output_string(&encoded);
        Ok(())
    }

    pub(crate) fn decode_payload_from_buffer(&mut self, len: usize) -> Result<(), String> {
        let raw = self.read_input_string(len)?;
        self.decode_full_code(&raw)
    }

    pub(crate) fn build_transport_frames_from_buffer(
        &mut self,
        len: usize,
        chunk_chars: usize,
    ) -> Result<usize, String> {
        let raw = self.read_input_string(len)?;
        self.frames = build_transport_frames(&raw, chunk_chars)?;
        Ok(self.frames.len())
    }

    pub(crate) fn export_transport_frame(&mut self, index: usize) -> bool {
        let Some(frame) = self.frames.get(index) else {
            return false;
        };
        let raw = frame.clone();
        self.set_output_string(&raw);
        true
    }

    pub(crate) fn transport_frame_count(&self) -> usize {
        self.frames.len()
    }

    pub(crate) fn reset_transport_assembly(&mut self) {
        self.assembly = None;
        self.received_parts = 0;
        self.total_parts = 0;
    }

    pub(crate) fn submit_transport_text_from_buffer(
        &mut self,
        len: usize,
    ) -> Result<TransportSubmitResult, String> {
        let raw = self.read_input_string(len)?;
        self.submit_transport_text(&raw)
    }

    pub(crate) fn transport_received_parts(&self) -> usize {
        self.received_parts
    }

    pub(crate) fn transport_total_parts(&self) -> usize {
        self.total_parts
    }

    fn decode_full_code(&mut self, raw: &str) -> Result<(), String> {
        let decoded = decode_pair_payload_to_json(raw)?;
        self.set_output_string(raw);
        self.set_decoded_payload_json(&decoded);
        Ok(())
    }

    fn submit_transport_text(&mut self, raw: &str) -> Result<TransportSubmitResult, String> {
        if let Some(frame) = parse_transport_frame(raw)? {
            let assembly = self
                .assembly
                .take()
                .filter(|assembly| {
                    assembly.invitation_id == frame.invitation_id
                        && assembly.kind == frame.kind
                        && assembly.count == frame.count
                })
                .unwrap_or_else(|| TransportAssembly {
                    invitation_id: frame.invitation_id.clone(),
                    kind: frame.kind,
                    count: frame.count,
                    parts: vec![None; frame.count],
                });

            if frame.index == 0 || frame.index > assembly.count {
                self.assembly = Some(assembly);
                return Err("QR chunk index out of range.".to_string());
            }

            let mut assembly = assembly;
            assembly.parts[frame.index - 1] = Some(frame.chunk);
            let received = assembly.parts.iter().filter(|part| part.is_some()).count();
            self.received_parts = received;
            self.total_parts = assembly.count;

            if received == assembly.count {
                let full_code = assembly
                    .parts
                    .iter()
                    .map(|part| part.as_deref().unwrap_or_default())
                    .collect::<String>();
                match self.decode_full_code(&full_code) {
                    Ok(()) => {
                        self.assembly = None;
                        return Ok(TransportSubmitResult::Complete);
                    }
                    Err(error) => {
                        self.assembly = Some(assembly);
                        return Err(error);
                    }
                }
            }

            self.assembly = Some(assembly);
            return Ok(TransportSubmitResult::Partial);
        }

        self.decode_full_code(raw)?;
        self.reset_transport_assembly();
        Ok(TransportSubmitResult::DirectCode)
    }
}

fn pair_prefix_for(kind: &str) -> Option<&'static str> {
    match kind {
        "mazocarta_offer" => Some(PAIR_PREFIX_OFFER_V1),
        "mazocarta_answer" => Some(PAIR_PREFIX_ANSWER_V1),
        _ => None,
    }
}

fn pair_prefix_metadata(prefix: &str) -> Option<(&'static str, &'static str, u8)> {
    match prefix {
        PAIR_PREFIX_OFFER_V1 => Some(("mazocarta_offer", "offer", 1)),
        PAIR_PREFIX_ANSWER_V1 => Some(("mazocarta_answer", "answer", 1)),
        _ => None,
    }
}

fn normalize_pair_payload_text(raw: &str) -> &str {
    raw.trim_matches(|ch| matches!(ch, '\r' | '\n' | '\t'))
}

fn base45_value(ch: u8) -> Option<u32> {
    BASE45_ALPHABET
        .iter()
        .position(|candidate| *candidate == ch)
        .map(|index| index as u32)
}

fn bytes_to_base45(bytes: &[u8]) -> String {
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        if index + 1 < bytes.len() {
            let value = bytes[index] as u32 * 256 + bytes[index + 1] as u32;
            output.push(BASE45_ALPHABET[(value % 45) as usize] as char);
            output.push(BASE45_ALPHABET[((value / 45) % 45) as usize] as char);
            output.push(BASE45_ALPHABET[(value / (45 * 45)) as usize] as char);
            index += 2;
        } else {
            let value = bytes[index] as u32;
            output.push(BASE45_ALPHABET[(value % 45) as usize] as char);
            output.push(BASE45_ALPHABET[(value / 45) as usize] as char);
            index += 1;
        }
    }
    output
}

fn base45_to_bytes(raw: &str) -> Result<Vec<u8>, String> {
    let bytes = raw.as_bytes();
    let mut output = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let first =
            base45_value(bytes[index]).ok_or_else(|| "Invalid Base45 payload.".to_string())?;
        let second = bytes
            .get(index + 1)
            .and_then(|value| base45_value(*value))
            .ok_or_else(|| "Invalid Base45 payload.".to_string())?;
        if index + 2 < bytes.len() {
            let third = base45_value(bytes[index + 2])
                .ok_or_else(|| "Invalid Base45 payload.".to_string())?;
            let value = first + second * 45 + third * 45 * 45;
            if value > 0xffff {
                return Err("Base45 payload overflow.".to_string());
            }
            output.push((value / 256) as u8);
            output.push((value % 256) as u8);
            index += 3;
        } else {
            let value = first + second * 45;
            if value > 0xff {
                return Err("Base45 payload overflow.".to_string());
            }
            output.push(value as u8);
            index += 2;
        }
    }
    Ok(output)
}

fn deflate_raw(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .map_err(|_| "Could not compress pairing payload.".to_string())?;
    encoder
        .finish()
        .map_err(|_| "Could not compress pairing payload.".to_string())
}

fn inflate_raw(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = DeflateDecoder::new(bytes);
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|_| "Compressed pairing code could not be restored.".to_string())?;
    Ok(output)
}

pub(crate) fn encode_pair_payload_from_json(raw_json: &str) -> Result<String, String> {
    let payload: PairPayloadInput =
        serde_json::from_str(raw_json).map_err(|_| "Invalid pairing payload.".to_string())?;
    let invitation_id = payload.invitation_id.trim().to_ascii_uppercase();
    if invitation_id.is_empty() || payload.description.sdp.is_empty() {
        return Err("Invalid pairing payload.".to_string());
    }
    encode_pair_payload_v1(&payload.kind, &invitation_id, &payload.description.sdp)
}

fn encode_pair_payload_v1(kind: &str, invitation_id: &str, sdp: &str) -> Result<String, String> {
    let Some(prefix) = pair_prefix_for(kind) else {
        return Err("Invalid pairing payload.".to_string());
    };
    let raw_bytes = sdp.as_bytes();
    let compressed = deflate_raw(raw_bytes)?;
    let (codec, encoded_bytes) = if compressed.len() + 4 < raw_bytes.len() {
        (PAIR_CODEC_COMPRESSED, compressed)
    } else {
        (PAIR_CODEC_PLAIN, raw_bytes.to_vec())
    };
    Ok(format!(
        "{prefix}:{invitation_id}:{codec}:{}",
        bytes_to_base45(&encoded_bytes)
    ))
}

pub(crate) fn decode_pair_payload_to_json(raw: &str) -> Result<String, String> {
    let raw = normalize_pair_payload_text(raw);
    if raw.is_empty() {
        return Err("Missing pairing payload.".to_string());
    }
    if raw.starts_with('{') {
        let payload: serde_json::Value =
            serde_json::from_str(raw).map_err(|_| "Invalid pairing payload.".to_string())?;
        return serde_json::to_string(&payload).map_err(|_| "Invalid pairing payload.".to_string());
    }
    let first_separator = raw
        .find(':')
        .ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let second_separator = raw[first_separator + 1..]
        .find(':')
        .map(|index| index + first_separator + 1)
        .ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let prefix = &raw[..first_separator];
    let invitation_id = &raw[first_separator + 1..second_separator];
    let (kind, description_type, version) =
        pair_prefix_metadata(prefix).ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let third_separator = raw[second_separator + 1..]
        .find(':')
        .map(|index| index + second_separator + 1)
        .ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let codec = raw[second_separator + 1..third_separator]
        .chars()
        .next()
        .ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let encoded_payload = &raw[third_separator + 1..];
    let bytes = base45_to_bytes(encoded_payload)?;
    let sdp_bytes = match codec {
        PAIR_CODEC_COMPRESSED => inflate_raw(&bytes)?,
        PAIR_CODEC_PLAIN => bytes,
        _ => return Err("Unsupported pairing payload codec.".to_string()),
    };
    let description = PairDescription {
        description_type: description_type.to_string(),
        sdp: String::from_utf8(sdp_bytes)
            .map_err(|_| "Compressed pairing code could not be restored.".to_string())?,
    };

    let payload = PairPayloadOutput {
        kind: kind.to_string(),
        version,
        invitation_id: invitation_id.to_string(),
        description,
    };
    serde_json::to_string(&payload).map_err(|_| "Invalid pairing payload.".to_string())
}

fn build_transport_frames(raw_code: &str, chunk_chars: usize) -> Result<Vec<String>, String> {
    let raw_code = normalize_pair_payload_text(raw_code);
    if raw_code.is_empty() {
        return Err("Missing pairing payload.".to_string());
    }
    let first_separator = raw_code
        .find(':')
        .ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let second_separator = raw_code[first_separator + 1..]
        .find(':')
        .map(|index| index + first_separator + 1)
        .ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let prefix = &raw_code[..first_separator];
    let invitation_id = &raw_code[first_separator + 1..second_separator];
    let (kind, _, _) =
        pair_prefix_metadata(prefix).ok_or_else(|| "Unrecognized pairing payload.".to_string())?;
    let frame_kind = if kind == "mazocarta_offer" { 'O' } else { 'R' };
    let chunk_chars = chunk_chars.max(1);
    let chars: Vec<char> = raw_code.chars().collect();
    let mut frames = Vec::new();
    let total = chars.len().div_ceil(chunk_chars);
    for index in 0..total {
        let start = index * chunk_chars;
        let end = ((index + 1) * chunk_chars).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        let chunk_len = chunk.chars().count();
        frames.push(format!(
            "{PAIR_TRANSPORT_PREFIX}:{invitation_id}:D:{frame_kind}:{}:{total}:{chunk_len}:{chunk}",
            index + 1
        ));
    }
    if frames.is_empty() {
        frames.push(format!(
            "{PAIR_TRANSPORT_PREFIX}:{invitation_id}:D:{frame_kind}:1:1:0:"
        ));
    }
    Ok(frames)
}

fn parse_transport_frame(raw: &str) -> Result<Option<TransportFrame>, String> {
    let text = normalize_pair_payload_text(raw);
    let Some(frame_body) = text
        .strip_prefix(PAIR_TRANSPORT_PREFIX)
        .and_then(|rest| rest.strip_prefix(':'))
    else {
        return Ok(None);
    };
    let mut parts = frame_body.splitn(7, ':');
    let invitation_id = parts
        .next()
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?;
    let frame_type = parts
        .next()
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?;
    let kind = parts
        .next()
        .and_then(|value| value.chars().next())
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?;
    let index = parts
        .next()
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?
        .parse::<usize>()
        .map_err(|_| "Malformed QR transport frame.".to_string())?;
    let count = parts
        .next()
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?
        .parse::<usize>()
        .map_err(|_| "Malformed QR transport frame.".to_string())?;
    let chunk_len = parts
        .next()
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?
        .parse::<usize>()
        .map_err(|_| "Malformed QR transport frame.".to_string())?;
    let chunk = parts
        .next()
        .ok_or_else(|| "Malformed QR transport frame.".to_string())?;

    if invitation_id.is_empty()
        || frame_type != "D"
        || !matches!(kind, 'O' | 'R')
        || index == 0
        || count == 0
        || index > count
    {
        return Err("Malformed QR transport frame.".to_string());
    }

    let received_len = chunk.chars().count();
    if received_len > chunk_len {
        return Err("Malformed QR transport frame.".to_string());
    }
    let mut chunk = chunk.to_string();
    if received_len < chunk_len {
        chunk.extend(std::iter::repeat_n(' ', chunk_len - received_len));
    }

    Ok(Some(TransportFrame {
        invitation_id: invitation_id.to_string(),
        kind,
        index,
        count,
        chunk,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_offer_sdp() -> &'static str {
        "v=0\r\n\
o=- 111111111111111111 1 IN IP4 192.0.2.1\r\n\
s=-\r\n\
t=0 0\r\n\
a=group:BUNDLE 0\r\n\
a=extmap-allow-mixed\r\n\
a=msid-semantic: WMS\r\n\
m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
c=IN IP4 0.0.0.0\r\n\
a=candidate:1111111111 1 udp 1111111111 192.0.2.55 55555 typ host generation 0 network-cost 10\r\n\
a=ice-ufrag:FAKE\r\n\
a=ice-pwd:FAKEICEPASSWORDFAKEICE0000\r\n\
a=ice-options:trickle\r\n\
a=fingerprint:sha-256 AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA:AA\r\n\
a=setup:actpass\r\n\
a=mid:0\r\n\
a=sctp-port:5000\r\n\
a=max-message-size:262144\r\n"
    }

    fn submit_raw(bridge: &mut PairingBridge, raw: &str) -> Result<TransportSubmitResult, String> {
        bridge.input_buffer = raw.as_bytes().to_vec();
        bridge.submit_transport_text_from_buffer(raw.len())
    }

    #[test]
    fn v1_pair_payload_round_trips_full_sdp() {
        let raw = serde_json::json!({
            "kind": "mazocarta_offer",
            "invitationId": "ABCD1234",
            "description": {
                "type": "offer",
                "sdp": sample_offer_sdp(),
            }
        })
        .to_string();

        let encoded = encode_pair_payload_from_json(&raw).expect("encode");
        assert!(encoded.starts_with("MZO1:ABCD1234:"));

        let decoded = decode_pair_payload_to_json(&encoded).expect("decode");
        let payload: serde_json::Value = serde_json::from_str(&decoded).expect("json");
        assert_eq!(payload["kind"], "mazocarta_offer");
        assert_eq!(payload["version"], 1);
        assert_eq!(payload["invitationId"], "ABCD1234");
        assert_eq!(payload["description"]["type"], "offer");
        assert_eq!(payload["description"]["sdp"], sample_offer_sdp());
    }

    #[test]
    fn transport_frames_reassemble_with_duplicates_and_out_of_order_frames() {
        let raw = serde_json::json!({
            "kind": "mazocarta_offer",
            "invitationId": "ABCD1234",
            "description": {
                "type": "offer",
                "sdp": sample_offer_sdp(),
            }
        })
        .to_string();
        let encoded = encode_pair_payload_from_json(&raw).expect("encode");
        let frames = build_transport_frames(&encoded, 24).expect("frames");
        assert!(frames.len() > 1);

        let mut bridge = PairingBridge::default();
        let mut order = vec![1usize, 0, 1];
        order.extend(2..frames.len());
        let mut last_result = None;
        for index in order {
            if index >= frames.len() {
                continue;
            }
            let result = submit_raw(&mut bridge, &frames[index]).expect("submit");
            last_result = Some(result);
        }
        match last_result.expect("result") {
            TransportSubmitResult::Complete => {
                assert_eq!(
                    std::str::from_utf8(&bridge.output_buffer).expect("utf8"),
                    encoded
                );
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn transport_assembly_survives_invalid_text_between_frames() {
        let raw = serde_json::json!({
            "kind": "mazocarta_offer",
            "invitationId": "ABCD1234",
            "description": {
                "type": "offer",
                "sdp": sample_offer_sdp(),
            }
        })
        .to_string();
        let encoded = encode_pair_payload_from_json(&raw).expect("encode");
        let frames = build_transport_frames(&encoded, 24).expect("frames");
        assert!(frames.len() > 1);

        let mut bridge = PairingBridge::default();
        assert!(matches!(
            submit_raw(&mut bridge, &frames[0]).expect("first frame"),
            TransportSubmitResult::Partial
        ));
        assert_eq!(bridge.transport_received_parts(), 1);
        assert_eq!(bridge.transport_total_parts(), frames.len());

        assert!(submit_raw(&mut bridge, "not a pairing code").is_err());
        assert_eq!(bridge.transport_received_parts(), 1);
        assert_eq!(bridge.transport_total_parts(), frames.len());

        assert!(submit_raw(&mut bridge, "MZQ1:malformed").is_err());
        assert_eq!(bridge.transport_received_parts(), 1);
        assert_eq!(bridge.transport_total_parts(), frames.len());

        let mut last_result = None;
        for frame in frames.iter().skip(1) {
            last_result = Some(submit_raw(&mut bridge, frame).expect("submit remaining"));
        }
        assert!(matches!(
            last_result.expect("result"),
            TransportSubmitResult::Complete
        ));
        assert_eq!(
            std::str::from_utf8(&bridge.output_buffer).expect("utf8"),
            encoded
        );
    }

    #[test]
    fn transport_frame_from_new_invitation_restarts_assembly() {
        let first_raw = serde_json::json!({
            "kind": "mazocarta_offer",
            "invitationId": "ABCD1234",
            "description": {
                "type": "offer",
                "sdp": sample_offer_sdp(),
            }
        })
        .to_string();
        let second_raw = serde_json::json!({
            "kind": "mazocarta_offer",
            "invitationId": "WXYZ9876",
            "description": {
                "type": "offer",
                "sdp": sample_offer_sdp(),
            }
        })
        .to_string();
        let first_encoded = encode_pair_payload_from_json(&first_raw).expect("encode first");
        let second_encoded = encode_pair_payload_from_json(&second_raw).expect("encode second");
        let first_frames = build_transport_frames(&first_encoded, 24).expect("first frames");
        let second_frames = build_transport_frames(&second_encoded, 24).expect("second frames");
        assert!(first_frames.len() > 1);
        assert!(second_frames.len() > 1);

        let mut bridge = PairingBridge::default();
        assert!(matches!(
            submit_raw(&mut bridge, &first_frames[0]).expect("first frame"),
            TransportSubmitResult::Partial
        ));
        assert_eq!(bridge.transport_received_parts(), 1);
        assert_eq!(bridge.transport_total_parts(), first_frames.len());

        assert!(matches!(
            submit_raw(&mut bridge, &second_frames[0]).expect("new first frame"),
            TransportSubmitResult::Partial
        ));
        assert_eq!(bridge.transport_received_parts(), 1);
        assert_eq!(bridge.transport_total_parts(), second_frames.len());

        let mut last_result = None;
        for frame in second_frames.iter().skip(1) {
            last_result = Some(submit_raw(&mut bridge, frame).expect("submit second"));
        }
        assert!(matches!(
            last_result.expect("result"),
            TransportSubmitResult::Complete
        ));
        assert_eq!(
            std::str::from_utf8(&bridge.output_buffer).expect("utf8"),
            second_encoded
        );
    }

    #[test]
    fn transport_frame_parser_restores_trailing_spaces_trimmed_by_decoders() {
        let frame = parse_transport_frame("MZQ1:ABCD1234:D:R:1:1:9:PAYLOAD:")
            .expect("parse")
            .expect("frame");

        assert_eq!(frame.invitation_id, "ABCD1234");
        assert_eq!(frame.kind, 'R');
        assert_eq!(frame.index, 1);
        assert_eq!(frame.count, 1);
        assert_eq!(frame.chunk, "PAYLOAD: ");
    }

    #[test]
    fn transport_encoder_uses_length_guarded_frames() {
        let raw_code = "MZO1:ABCD1234:P:PAYLOAD: ";
        let frames = build_transport_frames(raw_code, 64).expect("frames");
        assert_eq!(frames.len(), 1);
        assert!(frames[0].starts_with("MZQ1:ABCD1234:D:O:1:1:25:"));

        let trimmed = frames[0].trim_end_matches(' ');
        let frame = parse_transport_frame(trimmed)
            .expect("parse")
            .expect("frame");
        assert_eq!(frame.chunk, raw_code);
    }

    #[test]
    fn v1_answer_payload_round_trips_non_minimal_sdp() {
        let sdp = "v=0\r\no=- 1 1 IN IP4 192.0.2.1\r\ns=-\r\nt=0 0\r\nm=application 9 UDP/DTLS/SCTP other-kind\r\n";
        let payload = serde_json::json!({
            "kind": "mazocarta_answer",
            "invitationId": "DEADBEEF",
            "description": {
                "type": "answer",
                "sdp": sdp,
            }
        })
        .to_string();
        let encoded = encode_pair_payload_from_json(&payload).expect("encode");
        assert!(encoded.starts_with("MZA1:DEADBEEF:"));
        let decoded = decode_pair_payload_to_json(&encoded).expect("decode");
        let json: serde_json::Value = serde_json::from_str(&decoded).expect("json");
        assert_eq!(json["kind"], "mazocarta_answer");
        assert_eq!(json["version"], 1);
        assert_eq!(json["invitationId"], "DEADBEEF");
        assert_eq!(json["description"]["type"], "answer");
        assert_eq!(json["description"]["sdp"], sdp);
    }
}
