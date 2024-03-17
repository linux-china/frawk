use std::collections::{BTreeMap};
use jwt::{AlgorithmType, Header, SignWithKey, VerifyWithKey, Token, FromBase64};
use std::io::{BufReader, Cursor};
use sha2::{Sha256, Sha512, Digest, Sha384};
use hmac::{Hmac, Mac};
use jwt::header::HeaderType;
use serde_json::{Number, Value};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};

use crate::runtime::{SharedMap, Str, StrMap};

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128Enc>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128Dec>;

/// Message Digest with md5, sha256, sha512
pub fn digest(algorithm: &str, text: &str) -> String {
    if algorithm == "md5" || algorithm == "md-5" {
        return format!("{:x}", md5::compute(text));
    } else if algorithm == "adler32" {
        return adler::adler32(BufReader::new("demo2".as_bytes())).unwrap().to_string();
    } else if algorithm == "crc32" {
        return crc::Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(text.as_bytes()).to_string();
    } else if algorithm == "blake3" {
        return blake3::hash(text.as_bytes()).to_string();
    } else if algorithm == "sha256" || algorithm == "sha-256" {
        let mut hasher = Sha256::default();
        hasher.update(text.as_bytes());
        return format!("{:x}", hasher.finalize());
    } else if algorithm == "sha512" || algorithm == "sha-512" {
        let mut hasher = Sha512::default();
        hasher.update(text.as_bytes());
        return format!("{:x}", hasher.finalize());
    } else if algorithm == "bcrypt" {
        return bcrypt::hash(text, bcrypt::DEFAULT_COST).unwrap();
    } else if algorithm == "murmur3" {
        let hashcode = murmur3::murmur3_32(&mut Cursor::new(text), 0).unwrap();
        return hashcode.to_string();
    } else if algorithm == "xxh32" {
        return xxhash_rust::xxh32::xxh32(text.as_bytes(), 0).to_string();
    } else if algorithm == "xxh64" {
        return xxhash_rust::xxh64::xxh64(text.as_bytes(), 0).to_string();
    }
    format!("{}:{}", algorithm, text)
}

/// HMAC(Hash-based message authentication code) with HmacSHA256 and HmacSHA512
pub fn hmac(algorithm: &str, key: &str, text: &str) -> String {
    return if algorithm == "HmacSHA512" {
        let mut mac = HmacSha512::new_from_slice(key.as_bytes()).unwrap();
        mac.update(text.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    } else {
        let mut mac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
        mac.update(text.as_bytes());
        format!("{:x}", mac.finalize().into_bytes())
    };
}

pub(crate) fn jwt<'a>(algorithm: &str, key: &str, payload: &StrMap<'a, Str<'a>>) -> String {
    let mut claims: BTreeMap<String, Value> = BTreeMap::new();
    payload.iter(|map| {
        for (key, value) in map {
            let key = key.to_string();
            let value = value.to_string();
            if key == "exp" || key == "nbf" || key == "iat" {
                claims.insert(key, Value::Number(Number::from(value.parse::<u64>().unwrap())));
            } else {
                if let Ok(value) = value.parse::<i64>() {
                    claims.insert(key, Value::Number(Number::from(value)));
                } else if let Ok(value) = value.parse::<f64>() {
                    claims.insert(key, Value::Number(Number::from_f64(value).unwrap()));
                } else {
                    claims.insert(key, Value::String(value));
                }
            }
        }
    });
    let algorithm = algorithm.to_uppercase();
    let mut header = Header {
        type_: Some(HeaderType::JsonWebToken),
        ..Default::default()
    };
    if algorithm == "HS512" {
        let key = Hmac::<Sha512>::new_from_slice(key.as_bytes()).unwrap();
        header.algorithm = AlgorithmType::Hs512;
        Token::new(header, claims).sign_with_key(&key).unwrap()
    } else if algorithm == "HS384" {
        let key = Hmac::<Sha384>::new_from_slice(key.as_bytes()).unwrap();
        header.algorithm = AlgorithmType::Hs384;
        Token::new(header, claims).sign_with_key(&key).unwrap()
    } else {
        let key = Hmac::<Sha256>::new_from_slice(key.as_bytes()).unwrap();
        header.algorithm = AlgorithmType::Hs256;
        Token::new(header, claims).sign_with_key(&key).unwrap()
    }.as_str().to_string()
}

pub(crate) fn dejwt<'a>(key: &str, token: &str) -> StrMap<'a, Str<'a>> {
    let header_text = token[0..token.find('.').unwrap()].to_string();
    let header = Header::from_base64(&header_text).unwrap();
    let mut map = hashbrown::HashMap::new();
    let claims: BTreeMap<String, Value> = match header.algorithm {
        AlgorithmType::Hs256 => {
            let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).unwrap();
            token.verify_with_key(&key).unwrap()
        }
        AlgorithmType::Hs384 => {
            let key: Hmac<Sha384> = Hmac::new_from_slice(key.as_bytes()).unwrap();
            token.verify_with_key(&key).unwrap()
        }
        AlgorithmType::Hs512 => {
            let key: Hmac<Sha512> = Hmac::new_from_slice(key.as_bytes()).unwrap();
            token.verify_with_key(&key).unwrap()
        }
        _ => {
            BTreeMap::new()
        }
    };
    for (key, value) in claims {
        match value {
            Value::Null => {}
            Value::Bool(bool_value) => {
                if bool_value {
                    map.insert(Str::from(key), Str::from("1".to_string()));
                } else {
                    map.insert(Str::from(key), Str::from("0".to_string()));
                }
            }
            Value::Number(num) => {
                map.insert(Str::from(key), Str::from(num.to_string()));
            }
            Value::String(text) => {
                map.insert(Str::from(key), Str::from(text));
            }
            Value::Array(arr) => {
                map.insert(Str::from(key), Str::from(serde_json::to_string(&arr).unwrap()));
            }
            Value::Object(obj) => {
                map.insert(Str::from(key), Str::from(serde_json::to_string(&obj).unwrap()));
            }
        }
    }
    SharedMap::from(map)
}

/// plaintext max length 256
pub fn encrypt(_mode: &str, plaintext: &str, key_pass: &str, iv_text: &str) -> String {
    let mut key = [0x0; 16];
    let mut iv = [0x0; 16];
    if key_pass.len() > 16 {
        key.copy_from_slice(key_pass[..16].as_bytes());
    } else {
        key[..key_pass.len()].copy_from_slice(key_pass.as_bytes());
    }
    if !iv_text.is_empty() {
        let bytes = hex::decode(iv_text).unwrap();
        iv[..bytes.len()].copy_from_slice(&bytes);
    }
    // buffer must be big enough for padded plaintext
    let mut buf = [0u8; 512];
    let pt_len = plaintext.len();
    buf[..pt_len].copy_from_slice(plaintext.as_bytes());
    if _mode == "aes-128-gcm" {
        use aes_gcm::{aead::{Aead, KeyInit}, Aes128Gcm, Nonce};
        let cipher = Aes128Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&iv[..12]);
        let result = cipher.encrypt(&nonce, plaintext.as_bytes()).unwrap();
        hex::encode(&result)
    } else {
        let cipher = Aes128CbcEnc::new(&key.into(), &iv.into());
        let ct = cipher.encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len).unwrap();
        hex::encode(&ct)
    }
}

pub fn decrypt(_mode: &str, encrypted_text: &str, key_pass: &str, iv_text: &str) -> String {
    let mut key = [0x0; 16];
    let mut iv = [0x0; 16];
    if key_pass.len() > 16 {
        key.copy_from_slice(key_pass[..16].as_bytes());
    } else {
        key[..key_pass.len()].copy_from_slice(key_pass.as_bytes());
    }
    if !iv_text.is_empty() {
        let bytes = hex::decode(iv_text).unwrap();
        iv[..bytes.len()].copy_from_slice(&bytes);
    }
    let mut encrypted_data = hex::decode(encrypted_text).unwrap();
    if _mode == "aes-128-gcm" {
        use aes_gcm::{aead::{Aead, KeyInit}, Aes128Gcm, Nonce};
        let cipher = Aes128Gcm::new(&key.into());
        let nonce = Nonce::from_slice(&iv[0..12]);
        let pt = cipher.decrypt(nonce, encrypted_data.as_ref()).unwrap();
        std::str::from_utf8(&pt).unwrap().to_string()
    } else {
        let cipher = Aes128CbcDec::new(&key.into(), &iv.into());
        let pt = cipher.decrypt_padded_mut::<Pkcs7>(&mut encrypted_data).unwrap();
        std::str::from_utf8(pt).unwrap().to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use crate::runtime::encoding::encode;
    use super::*;

    #[test]
    fn test_md5() {
        let digest_message = digest("md5", "hello");
        println!("{}", digest_message);
    }

    #[test]
    fn test_sha_256() {
        let digest_message = digest("sha256", "hello");
        println!("{}", digest_message);
    }

    #[test]
    fn test_sha_512() {
        let digest_message = digest("sha512", "hello");
        println!("{}", digest_message);
    }

    #[test]
    fn test_hmac_sha_256() {
        let signature = hmac("HmacSha256", "7f4ebc75-7476-453e-b8d2-bebe17352b0a", "hello");
        println!("{}", signature);
    }

    #[test]
    fn test_jwt_hs256() {
        let header_payload = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ";
        let jwt_token = encode("hex-base64url", &hmac("HmacSha256", "123456", header_payload));
        println!("{}", jwt_token);
    }

    #[test]
    fn test_murmur3() {
        use std::io::Cursor;
        let hash_result = murmur3::murmur3_32(&mut Cursor::new("Hello"), 0);
        println!("{}", hash_result.unwrap());
    }

    #[test]
    fn test_xxh32() {
        let hash_result = xxhash_rust::xxh32::xxh32("hello".as_bytes(), 0).to_string();
        println!("{}", hash_result);
    }

    #[test]
    fn test_adler32() {
        let result = adler::adler32(BufReader::new("demo2".as_bytes())).unwrap();
        println!("{}", result);
    }

    #[test]
    fn test_crc32() {
        let result = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(b"123456789");
        println!("{}", result);
    }

    #[test]
    fn test_blake3() {
        println!("{}", digest("blake3", "demo"));
    }

    #[test]
    fn test_jwt() {
        let payload: StrMap<Str> = StrMap::default();
        payload.insert(Str::from("name"), Str::from("John Doe"));
        payload.insert(Str::from("user_uuid"), Str::from("8456ea54-62e8-4a31-9cce-18de7a6a890d"));
        payload.insert(Str::from("user_id"), Str::from("112344"));
        payload.insert(Str::from("rate"), Str::from("11.11"));
        payload.insert(Str::from("exp"), Str::from("1208234234234"));
        let token = jwt("HS256", "123456", &payload);
        println!("{}", token);
    }

    #[test]
    fn test_decode_head() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjEyMDgyMzQyMzQyMzQsIm5hbWUiOiJKb2huIERvZSIsInJhdGUiOjExLjExLCJ1c2VyX2lkIjoxMTIzNDQsInVzZXJfdXVpZCI6Ijg0NTZlYTU0LTYyZTgtNGEzMS05Y2NlLTE4ZGU3YTZhODkwZCJ9.P2e6b_I1pfbmgoyXcEwAKM1XjgNeRku0jatyf2CYD3o";
        let enc = token[0..token.find('.').unwrap()].to_string();
        let header = Header::from_base64(&enc).unwrap();
        println!("{:?}", header);
    }

    #[test]
    fn test_dejwt() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjEyMDgyMzQyMzQyMzQsIm5hbWUiOiJKb2huIERvZSIsInJhdGUiOjExLjExLCJ1c2VyX2lkIjoxMTIzNDQsInVzZXJfdXVpZCI6Ijg0NTZlYTU0LTYyZTgtNGEzMS05Y2NlLTE4ZGU3YTZhODkwZCJ9.P2e6b_I1pfbmgoyXcEwAKM1XjgNeRku0jatyf2CYD3o";
        let payload = dejwt("123456", token);
        let value = payload.get(&Str::from("exp"));
        println!("{}", value);
    }

    #[test]
    fn test_aes() {
        let key_pass = "0123456789abcdef";
        let iv_text = "2d069789e6dee8da14aa31b8";
        let plaintext = "Hello World";
        let encrypted_text = encrypt("aes-128-gcm", plaintext, key_pass, iv_text);
        println!("{}", encrypted_text);
        let plaintext2 = decrypt("aes-128-gcm", &encrypted_text, key_pass, iv_text);
        assert_eq!(plaintext, plaintext2);
    }
}