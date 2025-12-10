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
    // sequence control: 12 bit sequence number in the top bits, 4 bit fragment number below
    out[22..24].copy_from_slice(&((seq & 0x0fff) << 4).to_le_bytes());
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

pub trait Radio {
    fn send(&mut self, frame: &[u8]);
    fn recv(&mut self, buf: &mut [u8]) -> Option<usize>;
}

/// Radio with nothing behind it; used until a real backend exists.
pub struct NullRadio;

impl Radio for NullRadio {
    fn send(&mut self, _frame: &[u8]) {}
    fn recv(&mut self, _buf: &mut [u8]) -> Option<usize> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Scanning,
    Authenticating(Mac),
    Associating(Mac),
    Associated { bssid: Mac, aid: u16 },
    Failed,
}

pub struct Station<R: Radio> {
    radio: R,
    pub mac: Mac,
    pub state: State,
    ssid: [u8; 32],
    ssid_len: usize,
    seq: u16,
}

impl<R: Radio> Station<R> {
    pub fn new(radio: R, mac: Mac) -> Station<R> {
        Station { radio, mac, state: State::Idle, ssid: [0; 32], ssid_len: 0, seq: 0 }
    }

    fn next_seq(&mut self) -> u16 {
        self.seq = (self.seq + 1) & 0x0fff;
        self.seq
    }

    pub fn connect(&mut self, ssid: &[u8]) {
        let n = ssid.len().min(32);
        self.ssid[..n].copy_from_slice(&ssid[..n]);
        self.ssid_len = n;
        let mut buf = [0u8; 80];
        let seq = self.next_seq();
        if let Ok(len) = probe_request(&mut buf, &self.mac, &self.ssid[..n], seq) {
            self.radio.send(&buf[..len]);
            self.state = State::Scanning;
        }
    }

    /// Feeds one received frame to the state machine.
    pub fn handle(&mut self, frame: &[u8]) {
        let mut buf = [0u8; 80];
        let ssid_len = self.ssid_len;
        match (self.state, parse(frame)) {
            (State::Scanning, Ok(Rx::ProbeResponse { bssid, ssid })) if ssid == &self.ssid[..ssid_len] => {
                let seq = self.next_seq();
                if let Ok(n) = auth_request(&mut buf, &self.mac, &bssid, seq) {
                    self.radio.send(&buf[..n]);
                    self.state = State::Authenticating(bssid);
                }
            }
            (State::Authenticating(bssid), Ok(Rx::Auth { status })) => {
                if status != 0 {
                    self.state = State::Failed;
                    return;
                }
                let seq = self.next_seq();
                let ssid = self.ssid;
                if let Ok(n) = assoc_request(&mut buf, &self.mac, &bssid, &ssid[..ssid_len], seq) {
                    self.radio.send(&buf[..n]);
                    self.state = State::Associating(bssid);
                }
            }
            (State::Associating(bssid), Ok(Rx::AssocResponse { status, aid })) => {
                self.state = if status == 0 { State::Associated { bssid, aid } } else { State::Failed };
            }
            (_, Ok(Rx::Deauth)) => self.state = State::Idle,
            _ => {}
        }
    }

    /// Pulls frames from the radio until it has nothing more.
    pub fn poll(&mut self) {
        let mut buf = [0u8; 256];
        while let Some(n) = self.radio.recv(&mut buf) {
            let frame = buf;
            self.handle(&frame[..n]);
        }
    }

    pub fn state_code(&self) -> isize {
        match self.state {
            State::Idle | State::Failed => 0,
            State::Associated { .. } => 2,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::vec::Vec;

    const AP: Mac = [0x02, 0xaa, 0, 0, 0, 1];
    const STA: Mac = [0x02, 0, 0, 0, 0, 1];

    /// Access point answering probe, auth and assoc requests for one SSID.
    struct FakeAp {
        ssid: &'static [u8],
        inbox: VecDeque<Vec<u8>>,
        reject_auth: bool,
    }

    fn header(fc: u16, to: &Mac) -> Vec<u8> {
        let mut v = std::vec![0u8; HDR_LEN];
        put_header(&mut v, fc, to, &AP, &AP, 1);
        v
    }

    impl Radio for FakeAp {
        fn send(&mut self, frame: &[u8]) {
            let fc = u16::from_le_bytes([frame[0], frame[1]]);
            let mut sa = [0u8; 6];
            sa.copy_from_slice(&frame[10..16]);
            match fc {
                FC_PROBE_REQ => {
                    let mut r = header(FC_PROBE_RESP, &sa);
                    r.extend_from_slice(&[0; 12]);
                    r.push(0);
                    r.push(self.ssid.len() as u8);
                    r.extend_from_slice(self.ssid);
                    self.inbox.push_back(r);
                }
                FC_AUTH => {
                    let mut r = header(FC_AUTH, &sa);
                    r.extend_from_slice(&[0, 0, 2, 0, if self.reject_auth { 1 } else { 0 }, 0]);
                    self.inbox.push_back(r);
                }
                FC_ASSOC_REQ => {
                    let mut r = header(FC_ASSOC_RESP, &sa);
                    r.extend_from_slice(&[1, 0, 0, 0, 0x05, 0xc0]); // aid 5 with the two top bits set
                    self.inbox.push_back(r);
                }
                _ => {}
            }
        }
        fn recv(&mut self, buf: &mut [u8]) -> Option<usize> {
            let f = self.inbox.pop_front()?;
            buf[..f.len()].copy_from_slice(&f);
            Some(f.len())
        }
    }

    #[test]
    fn associates() {
        let ap = FakeAp { ssid: b"home", inbox: VecDeque::new(), reject_auth: false };
        let mut sta = Station::new(ap, STA);
        sta.connect(b"home");
        assert_eq!(sta.state, State::Scanning);
        sta.poll();
        assert_eq!(sta.state, State::Associated { bssid: AP, aid: 5 });
    }

    #[test]
    fn ignores_other_ssids() {
        let ap = FakeAp { ssid: b"neighbour", inbox: VecDeque::new(), reject_auth: false };
        let mut sta = Station::new(ap, STA);
        sta.connect(b"home");
        sta.poll();
        assert_eq!(sta.state, State::Scanning);
    }

    #[test]
    fn auth_rejection_fails() {
        let ap = FakeAp { ssid: b"home", inbox: VecDeque::new(), reject_auth: true };
        let mut sta = Station::new(ap, STA);
        sta.connect(b"home");
        sta.poll();
        assert_eq!(sta.state, State::Failed);
    }

    #[test]
    fn sequence_control_layout() {
        let mut b = [0u8; 40];
        probe_request(&mut b, &STA, b"x", 3).unwrap();
        assert_eq!(u16::from_le_bytes([b[22], b[23]]), 3 << 4);
    }

    #[test]
    fn short_frames_are_errors() {
        assert_eq!(parse(&[0u8; 10]), Err(FrameError::Short));
    }
}
