//! 802.11 station: management frame codec and the scan/auth/assoc state machine.
//! The radio is a trait so the state machine runs against a software access point in tests.
#![cfg_attr(not(test), no_std)]

pub const FC_ASSOC_REQ: u16 = 0x0000;
pub const FC_ASSOC_RESP: u16 = 0x0010;
pub const FC_PROBE_REQ: u16 = 0x0040;
pub const FC_PROBE_RESP: u16 = 0x0050;
pub const FC_AUTH: u16 = 0x00b0;
pub const FC_DEAUTH: u16 = 0x00c0;

pub const HDR_LEN: usize = 24;
pub type Mac = [u8; 6];
pub const BROADCAST: Mac = [0xff; 6];

#[derive(Debug, PartialEq)]
pub enum FrameError {
    Short,
    BufferTooSmall,
    NotManagement,
}

/// Writes the 24 byte management header. Returns HDR_LEN.
fn put_header(out: &mut [u8], fc: u16, da: &Mac, sa: &Mac, bssid: &Mac, seq: u16) -> usize {
    out[0..2].copy_from_slice(&fc.to_le_bytes());
    out[2..4].copy_from_slice(&0u16.to_le_bytes()); // duration
    out[4..10].copy_from_slice(da);
    out[10..16].copy_from_slice(sa);
    out[16..22].copy_from_slice(bssid);
    out[22..24].copy_from_slice(&seq.to_le_bytes());
    HDR_LEN
}

fn put_ie(out: &mut [u8], pos: usize, id: u8, data: &[u8]) -> usize {
    out[pos] = id;
    out[pos + 1] = data.len() as u8;
    out[pos + 2..pos + 2 + data.len()].copy_from_slice(data);
    pos + 2 + data.len()
}

const RATES: [u8; 4] = [0x82, 0x84, 0x8b, 0x96]; // 1, 2, 5.5, 11 Mbit/s, basic

pub fn probe_request(out: &mut [u8], sa: &Mac, ssid: &[u8], seq: u16) -> Result<usize, FrameError> {
    if out.len() < HDR_LEN + 2 + ssid.len() + 2 + RATES.len() {
        return Err(FrameError::BufferTooSmall);
    }
    let mut n = put_header(out, FC_PROBE_REQ, &BROADCAST, sa, &BROADCAST, seq);
    n = put_ie(out, n, 0, ssid);
    Ok(put_ie(out, n, 1, &RATES))
}

pub fn auth_request(out: &mut [u8], sa: &Mac, bssid: &Mac, seq: u16) -> Result<usize, FrameError> {
    if out.len() < HDR_LEN + 6 {
        return Err(FrameError::BufferTooSmall);
    }
    let n = put_header(out, FC_AUTH, bssid, sa, bssid, seq);
    out[n..n + 2].copy_from_slice(&0u16.to_le_bytes()); // open system
    out[n + 2..n + 4].copy_from_slice(&1u16.to_le_bytes()); // transaction 1
    out[n + 4..n + 6].copy_from_slice(&0u16.to_le_bytes()); // status
    Ok(n + 6)
}

pub fn assoc_request(out: &mut [u8], sa: &Mac, bssid: &Mac, ssid: &[u8], seq: u16) -> Result<usize, FrameError> {
    if out.len() < HDR_LEN + 4 + 2 + ssid.len() + 2 + RATES.len() {
        return Err(FrameError::BufferTooSmall);
    }
    let mut n = put_header(out, FC_ASSOC_REQ, bssid, sa, bssid, seq);
    out[n..n + 2].copy_from_slice(&0x0001u16.to_le_bytes()); // capability: ESS
    out[n + 2..n + 4].copy_from_slice(&10u16.to_le_bytes()); // listen interval
    n += 4;
    n = put_ie(out, n, 0, ssid);
    Ok(put_ie(out, n, 1, &RATES))
}

/// What the station needs to know about a received management frame.
#[derive(Debug, PartialEq)]
pub enum Rx<'a> {
    ProbeResponse { bssid: Mac, ssid: &'a [u8] },
    Auth { status: u16 },
    AssocResponse { status: u16, aid: u16 },
    Deauth,
    Other,
}

pub fn parse(frame: &[u8]) -> Result<Rx<'_>, FrameError> {
    if frame.len() < HDR_LEN {
        return Err(FrameError::Short);
    }
    let fc = u16::from_le_bytes([frame[0], frame[1]]);
    if fc & 0x000c != 0 {
        return Err(FrameError::NotManagement);
    }
    let body = &frame[HDR_LEN..];
    let mut bssid = [0u8; 6];
    bssid.copy_from_slice(&frame[16..22]);
    Ok(match fc & 0x00f0 | (fc & 0xff00) {
        FC_PROBE_RESP => {
            // fixed fields: timestamp(8) interval(2) capability(2), then information elements
            if body.len() < 14 {
                return Err(FrameError::Short);
            }
            let ies = &body[12..];
            let ssid = if ies.len() >= 2 && ies[0] == 0 && ies.len() >= 2 + ies[1] as usize {
                &ies[2..2 + ies[1] as usize]
            } else {
                &[]
            };
            Rx::ProbeResponse { bssid, ssid }
        }
        FC_AUTH => {
            if body.len() < 6 {
                return Err(FrameError::Short);
            }
            Rx::Auth { status: u16::from_le_bytes([body[4], body[5]]) }
        }
        FC_ASSOC_RESP => {
            if body.len() < 6 {
                return Err(FrameError::Short);
            }
            Rx::AssocResponse {
                status: u16::from_le_bytes([body[2], body[3]]),
                aid: u16::from_le_bytes([body[4], body[5]]) & 0x3fff, // top two bits are always set
            }
        }
        FC_DEAUTH => Rx::Deauth,
        _ => Rx::Other,
    })
}
