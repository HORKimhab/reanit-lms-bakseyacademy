//! Bakong KHQR (EMVCo merchant-presented QR) payloads: TLV fields, CRC-16/CCITT-FALSE checksum
//! (tag 63), and the MD5 Bakong uses to look a payment up.

/// The receiving Bakong account and how it is labelled in the payer's app.
#[derive(Clone, Debug)]
pub struct Merchant {
    /// Bakong account id, e.g. `reanit@aclb`.
    pub account_id: String,
    /// Shown to the payer; at most 25 characters.
    pub name: String,
    /// At most 15 characters.
    pub city: String,
    /// With an acquiring bank merchant id the QR uses the merchant template (tag 30), otherwise the
    /// individual one (tag 29).
    pub merchant_id: Option<String>,
    pub acquiring_bank: Option<String>,
}

/// One dynamic (amount-carrying) USD payment.
pub struct Payment<'a> {
    pub amount_cents: i64,
    /// Bill number shown in the payer's history (tag 62/01), at most 25 characters.
    pub bill_number: &'a str,
    pub created_ms: i64,
    pub expires_ms: i64,
}

/// One EMVCo TLV field: two-digit id, two-digit length (in characters), value.
fn tlv(id: &str, value: &str) -> String {
    format!("{id}{:02}{value}", value.chars().count())
}

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF) as four upper-case hex digits.
pub fn crc16(data: &str) -> String {
    let mut crc: u16 = 0xFFFF;
    for byte in data.bytes() {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    format!("{crc:04X}")
}

/// The full KHQR string for a dynamic USD payment, ending in its CRC.
pub fn build(merchant: &Merchant, payment: &Payment) -> String {
    let mut account = tlv("00", &merchant.account_id);
    let template = match (&merchant.merchant_id, &merchant.acquiring_bank) {
        (Some(id), Some(bank)) => {
            account.push_str(&tlv("01", id));
            account.push_str(&tlv("02", bank));
            "30"
        }
        _ => "29",
    };
    let amount = format!("{}.{:02}", payment.amount_cents / 100, payment.amount_cents % 100);
    let timestamps = tlv("00", &payment.created_ms.to_string()) + &tlv("01", &payment.expires_ms.to_string());
    let mut qr = [
        tlv("00", "01"),
        tlv("01", "12"),
        tlv(template, &account),
        tlv("52", "5999"),
        tlv("53", "840"),
        tlv("54", &amount),
        tlv("58", "KH"),
        tlv("59", &merchant.name),
        tlv("60", &merchant.city),
        tlv("62", &tlv("01", payment.bill_number)),
        tlv("99", &timestamps),
    ]
    .concat();
    qr.push_str("6304");
    let crc = crc16(&qr);
    qr + &crc
}

/// Bakong looks a payment up by the MD5 of the exact QR string the payer scanned.
pub fn md5_hex(qr: &str) -> String {
    use md5::{Digest, Md5};
    Md5::digest(qr.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merchant() -> Merchant {
        Merchant {
            account_id: "reanit@aclb".into(),
            name: "Reanit Academy".into(),
            city: "Phnom Penh".into(),
            merchant_id: None,
            acquiring_bank: None,
        }
    }

    const PAYMENT: Payment<'static> =
        Payment { amount_cents: 1550, bill_number: "AB12CD34", created_ms: 1759805345337, expires_ms: 1759806245337 };

    #[test]
    fn crc_matches_the_ccitt_false_check_value_and_the_official_khqr_example() {
        assert_eq!(crc16("123456789"), "29B1");
        // KHQR guideline example (developer.payway.com.kh): everything up to and including "6304".
        let official = concat!(
            "000201010212",
            "30510016abaakhppxxx@abaa01151250212145328460208ABA Bank",
            "52045987", "5303116", "5403100", "5802KH",
            "5925OLD ME 25 CHAR WINNER IP2", "6010Phnom Penh",
            "62570115MC-REF-KH-1500068340010PAYWAY@ABA0208104514230604A227",
            "99340013175980534533701131759805525337", "6304"
        );
        assert_eq!(crc16(official), "9FBD");
    }

    #[test]
    fn builds_an_individual_dynamic_usd_qr() {
        assert_eq!(
            build(&merchant(), &PAYMENT),
            concat!(
                "000201010212", "29150011reanit@aclb", "52045999", "5303840", "540515.50", "5802KH",
                "5914Reanit Academy", "6010Phnom Penh", "62120108AB12CD34",
                "99340013175980534533701131759806245337", "6304E457"
            )
        );
    }

    #[test]
    fn builds_a_merchant_qr_when_an_acquiring_bank_is_set() {
        let m = Merchant { merchant_id: Some("125021214532846".into()), acquiring_bank: Some("ACLEDA PL".into()), ..merchant() };
        let p = Payment { amount_cents: 100, ..PAYMENT };
        assert_eq!(
            build(&m, &p),
            concat!(
                "000201010212", "30470011reanit@aclb0115125021214532846", "0209ACLEDA PL", "52045999", "5303840",
                "54041.00", "5802KH", "5914Reanit Academy", "6010Phnom Penh", "62120108AB12CD34",
                "99340013175980534533701131759806245337", "63041661"
            )
        );
    }

    #[test]
    fn md5_is_lowercase_hex_of_the_whole_qr() {
        assert_eq!(md5_hex(&build(&merchant(), &PAYMENT)), "740eb6cb6611a4a711ce2ce4179eb838");
    }
}
