/// Concatenate heterogeneous byte slices into a single payload.
///
/// Every item must implement `AsRef<[u8]>` — this covers `Vec<u8>`,
/// `[u8; N]`, `&[u8]`, and the arrays returned by `p32` / `p64`.
///
/// ```
/// use pwnkit::{flat, p64};
///
/// let payload = flat![
///     b"A".repeat(40),
///     p64(0xdeadbeef),
///     b"BBBB".as_slice(),
/// ];
/// assert_eq!(payload.len(), 40 + 8 + 4);
/// ```
#[macro_export]
macro_rules! flat {
    ($($item:expr),* $(,)?) => {{
        let mut buf: Vec<u8> = Vec::new();
        $(buf.extend_from_slice(($item).as_ref());)*
        buf
    }};
}
