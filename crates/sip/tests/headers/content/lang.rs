use nirah_sip::Header;
use nirah_sip::headers::Language;
use nirah_sip::headers::parse::parse_content_language_header;

#[test]
fn write() {
    let header = Header::ContentLanguage(Language::English);
    assert_eq!("Content-Language: en".to_string(), format!("{}", header));
}

#[test]
fn read() {
    let remains = vec![];
    let header = Header::ContentLanguage(Language::English);
    assert_eq!(Ok((remains.as_ref(), header)), parse_content_language_header(b"Content-Language: en"));
}
