use chisel_decoders::selector::Encoding;
use chisel_json::{parsers::sax::Parser, parsers::sax_events::Match};

/// Macro to tidy up the match arm
macro_rules! selected_event {
    () => {
        Match::StartObject
    };
}

/// Extract all the unique *object* pointers from a given document, using the SAX parser and an appropriate set
/// of matching [Match] values
fn main() {
    let parser = Parser::with_encoding(Encoding::Utf8);
    let _result = parser.parse_file("fixtures/json/bench/simple.json", &mut |evt| {
        match evt.matched {
            selected_event!() => println!("{}", evt.pointer.unwrap()),
            _ => (),
        }
        Ok(())
    });
}
