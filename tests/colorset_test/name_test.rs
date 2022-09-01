use bytes::BytesMut;
use cls_rs::{colorset::name::*, utils::ExtendBytesMut};

#[test]
fn name_test() {
    let str0 = "\u{3400}test\u{1f5ff}set\u{0414}";
    let str1 = "\u{1f5ff}testset";
    let str3 = "ßtest\u{3400}";
    let str4 = std::iter::repeat("あ").take(64).collect::<String>();
    let mut str5 = std::iter::repeat("t").take(62).collect::<String>();
    str5.push('\u{1f5ff}');

    [str0, str1, str3, &str4, &str5]
        .into_iter()
        .map(|str| {
            let mut csn = ColorsetName::new();
            csn.set_str(str).unwrap();
            csn
        })
        .for_each(|csn| {
            let mut byte_csn = BytesMut::new();
            csn.extend_bytes(&mut byte_csn);

            let (_, de_csn) = bytes_colorset_name(byte_csn.as_ref()).unwrap();

            assert_eq!(de_csn, csn);
        });
}
