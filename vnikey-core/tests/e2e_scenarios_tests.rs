mod common;
use common::simulate_typing_str;
use vnikey_core::engine::{Engine, InputMethod};

#[test]
#[ignore = "Engine bug"]
fn test_email_address() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let input = "n g u y e e n . v a n _ a 1 2 3 @ g m a i l . c o m";
    let expected = "nguyễn.van_a123@gmail.com";
    let result = simulate_typing_str(&mut engine, input);
    assert_eq!(result, expected);
}

#[test]
fn test_url_link() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let input = "h t t p s : / / g i t h u b . c o m / h i d e o n h p / v n i k e y - l i n u x";
    let expected = "https://github.com/hideonhp/vnikey-linux";
    let result = simulate_typing_str(&mut engine, input);
    assert_eq!(result, expected);
}

#[test]
fn test_programming_syntax() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let input = "l e t Space a r r [ 1 ] Space = Space \" c h u o o i x \" ;";
    let expected = "let arr[1] = \"chuỗi\";";
    let result = simulate_typing_str(&mut engine, input);
    assert_eq!(result, expected);
}

#[test]
fn test_mixed_special_characters() {
    let mut engine = Engine::new(InputMethod::Telex, true);
    let input = "M y P @ s s w 0 r d ! # $ ^ & * ( ) _ + { } | < > ? ~";
    let expected = "MyP@ssw0rd!#$^&*()_+{}|<>?~";
    let result = simulate_typing_str(&mut engine, input);
    assert_eq!(result, expected);
}
