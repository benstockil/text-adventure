use crate::StoryEvent;
use peg;

peg::parser! { pub grammar story_parser() for str {
    // Define whitespace
    rule _ = quiet!{[' ' | '\t']+}

    rule eof() = ![_]

    rule event() -> StoryEvent = s:command() / s:text() / expected!("event")
    
    // Commands are uppercase words preceded by a plus sign, and can have arguments
    rule command() -> StoryEvent
        = "+" cmd:$(['A'..='Z']+) _? arg:arg()? { 
            match cmd {
                "PAUSE" => StoryEvent::Pause,
                "CLEAR" => StoryEvent::Clear,
                "INPUT" => StoryEvent::Input(arg.unwrap().to_owned()),
                _ => todo!(),
            }
        }
        / expected!("command")

    // Arguments terminate at whitespace / newline
    rule arg() -> &'input str
        = ":" _? arg:$((!("\n"/_)  [_])+) { arg }
        / expected!("argument")

    // rule args() -> Vec<String> 
    //     = args:( $([_]+) ++ (_* "," _*)) { args }

    // Text must terminate before the next command (plus sign)
    rule text() -> StoryEvent
        = t:$((!"\n+" [_])+) { StoryEvent::Text(t.to_owned()) }
        / expected!("text")

    pub rule story() -> Vec<StoryEvent>
        = l:(event() ** ("\n"+)) (_/"\n")* { l }
}}
