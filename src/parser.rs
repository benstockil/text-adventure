use crate::StoryEvent;
use peg;

// FIX: Parser cannot handle commands at EOF 

peg::parser! { pub grammar story_parser() for str {
    // Define whitespace
    rule _ = quiet!{[' ' | '\t']+}

    rule event() -> StoryEvent = s:command() / s:text()
    
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

    // Arguments terminate at whitespace / newline
    rule arg() -> &'input str
        = ":" _? arg:$((!("\n"/_)  [_])+) { arg }

    // rule args() -> Vec<String> 
    //     = args:( $([_]+) ++ (_* "," _*)) { args }

    // Text must terminate before the next command (plus sign)
    rule text() -> StoryEvent
        = t:$((!"\n+" [_])+) { StoryEvent::Text(t.to_owned()) }

    pub rule story() -> Vec<StoryEvent>
        = (event() ** ("\n"+))
}}
