use cls_rs::colorset::common::*;
use cls_rs::colorset::Colorset;

#[test]
fn colorset_test_with_read_file() {
    use std::env;
    use std::fs;

    let mut test_file_path = env::current_dir().unwrap();
    test_file_path.push("tests/colorset_test/testset.cls");

    let test_file_bytes = fs::read(test_file_path).unwrap();

    let (_, de_cls) = Colorset::try_from_bytes(&test_file_bytes).unwrap();

    //println!("{:?}", de_cls);

    let se_cls = de_cls.as_bytes();

    let se_cls_vec = se_cls.to_vec();

    assert_eq!(se_cls_vec, test_file_bytes);
}
