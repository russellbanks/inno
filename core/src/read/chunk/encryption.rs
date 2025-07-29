#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Encryption {
    #[default]
    Plaintext,
    Arc4Md5,
    Arc4Sha1,
    XChaCha20,
}
