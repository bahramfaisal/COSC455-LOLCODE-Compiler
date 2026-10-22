
grammar LOLCODEMarkdown;

lolcode
    : '#HAI' start '#KBYE' EOF
    ;

start
    : head? document
    ;

document
    : element*
    ;

element
    : comment
    | paragraph
    | bold
    | list
    | newline
    | link
    | variable_def
    | variable_use
    | text
    ;

comment
    : '#OBTW' text '#TLDR'
    ;

head
    : '#MAEK' 'HEAD' variable_def? title? '#MKAY'
    ;

title
    : '#GIMMEH' 'TITLE' text '#OIC'
    ;

paragraph
    : '#MAEK' 'PARAGRAF' variable_def? paragraph_element* '#MKAY'
    ;

paragraph_element
    : bold
    | list
    | newline
    | link
    | variable_use
    | text
    ;

bold
    : '#GIMMEH' 'BOLD' variable_def? text '#OIC'
    ;

list
    : '#MAEK' 'LIST' variable_def? item* '#MKAY'
    ;

item
    : '#GIMMEH' 'ITEM' variable_def? item_element* '#OIC'
    ;

item_element
    : bold
    | variable_use
    | text
    ;

newline
    : '#NEWLINE'
    ;

link
    : '#GIMMEH' 'LINX' text '#OIC'
    ;

variable_def
    : '#IHAZ' word '#ITIZ' text '#MKAY'
    ;

variable_use
    : '#GIMMEH' 'VAR' word '#OIC'
    ;

word
    : WORD
    ;

text
    : WORD
    | NUMBER
    | PUNCTUATION
    ;

WORD
    : ('A'..'Z' | 'a'..'z')+
    ;

NUMBER
    : ('0'..'9')+
    ;

PUNCTUATION
    : '.'
    | ','
    | '"'
    | ':'
    | '?'
    | '!'
    | '%'
    | '='
    | '/'
    ;

WS
    : (' ' | '\t' | '\r' | '\n')+ {skip();}
    ;
